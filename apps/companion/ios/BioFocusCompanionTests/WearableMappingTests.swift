import HealthKit
import XCTest

@testable import BioFocusCompanion

final class WearableMappingTests: XCTestCase {
    private let salt = Data("biofocus-test-salt".utf8)

    func testPairingQrKeepsPinAndLoopbackDash() {
        let pin = String(repeating: "ab", count: 32)
        let text = "biofocus:1\nhttps://10.0.0.8:8787/\n" + "tok\n" + pin + "\n"
        let parsed = PairingQr.parse(text)
        XCTAssertEqual(parsed?.url, "https://10.0.0.8:8787")
        XCTAssertEqual(parsed?.token, "tok")
        XCTAssertEqual(parsed?.pin, pin)
        let local = PairingQr.parse("biofocus:1\nhttp://127.0.0.1:8787\ntok\n-\n")
        XCTAssertNil(local?.pin)
    }

    func testPinMismatchOnCertificateBytes() {
        let der = Data("not-a-cert".utf8)
        let right = TlsPin.sha256Hex(der)
        XCTAssertTrue(TlsPin.matches(der: der, expectedHex: right))
        XCTAssertFalse(TlsPin.matches(der: der, expectedHex: String(repeating: "cd", count: 32)))
        XCTAssertFalse(TlsPin.matches(der: der, expectedHex: "abcd"))
    }

    func testPlainHttpOffLoopbackIsRejected() {
        let lan = URL(string: "http://192.168.1.20:8787")!
        XCTAssertThrowsError(try IngestURLPolicy.validate(url: lan, pin: "")) { error in
            XCTAssertEqual(error as? IngestClientError, .plainHttpOffLoopback)
        }
        let loopback = URL(string: "http://127.0.0.1:8787")!
        XCTAssertNoThrow(try IngestURLPolicy.validate(url: loopback, pin: ""))
        let https = URL(string: "https://192.168.1.20:8787")!
        XCTAssertThrowsError(try IngestURLPolicy.validate(url: https, pin: "  ")) { error in
            XCTAssertEqual(error as? IngestClientError, .pinRequired)
        }
    }

    func testKindTable() {
        XCTAssertEqual(
            WearableKind.classify(bundleId: "com.xiaomi.wearable", productType: nil, hardwareVersion: nil, wasUserEntered: false),
            "xiaomi_mi_fitness"
        )
        XCTAssertEqual(
            WearableKind.classify(bundleId: "com.huami.watch", productType: nil, hardwareVersion: nil, wasUserEntered: false),
            "zepp_life"
        )
        XCTAssertEqual(
            WearableKind.classify(bundleId: "com.xiaomi.hm.health", productType: nil, hardwareVersion: nil, wasUserEntered: false),
            "zepp_life"
        )
        XCTAssertEqual(
            WearableKind.classify(bundleId: "com.apple.health", productType: "Watch7,1", hardwareVersion: nil, wasUserEntered: false),
            "apple_watch"
        )
        XCTAssertEqual(
            WearableKind.classify(bundleId: "com.apple.health", productType: "iPhone17,1", hardwareVersion: nil, wasUserEntered: false),
            "iphone"
        )
        XCTAssertEqual(
            WearableKind.classify(bundleId: "com.apple.health", productType: "Watch7,1", hardwareVersion: nil, wasUserEntered: true),
            "manual"
        )
        XCTAssertEqual(
            WearableKind.classify(bundleId: "com.example.other", productType: nil, hardwareVersion: nil, wasUserEntered: false),
            "other_app"
        )
    }

    func testSleepStagesStaySeparate() {
        XCTAssertEqual(SleepStageMap.stage(hkValue: HKCategoryValueSleepAnalysis.asleepCore.rawValue), "asleep_core")
        XCTAssertEqual(SleepStageMap.stage(hkValue: HKCategoryValueSleepAnalysis.asleepDeep.rawValue), "asleep_deep")
        XCTAssertEqual(SleepStageMap.stage(hkValue: HKCategoryValueSleepAnalysis.asleepREM.rawValue), "asleep_rem")
        XCTAssertEqual(SleepStageMap.stage(hkValue: HKCategoryValueSleepAnalysis.asleepUnspecified.rawValue), "asleep")
        XCTAssertNotEqual(
            SleepStageMap.stage(hkValue: HKCategoryValueSleepAnalysis.asleepDeep.rawValue),
            "asleep"
        )
        XCTAssertEqual(SleepIntervalSample.mapStage(HKCategoryValueSleepAnalysis.asleepREM.rawValue), "asleep_rem")
    }

    func testHrvIsSdnnAndNeverRmssd() {
        let record = HealthRecord(
            uuid: "0190A030-0000-7000-8000-000000000003",
            identifier: HealthTypeRegistry.hrvSDNN,
            start: 1_772_406_000,
            end: 1_772_406_000,
            value: 48,
            categoryValue: nil,
            categoryName: nil,
            activityType: nil,
            kcal: nil,
            distanceMeters: nil,
            source: watchSource()
        )
        let object = WearableMapper.observationObject(for: record, salt: salt)
        let payload = object["payload"] as? [String: Any]
        XCTAssertEqual(payload?["method"] as? String, "sdnn")
        XCTAssertEqual(payload?["sdnn_ms"] as? Int, 48)
        XCTAssertNil(payload?["rmssd_ms"])
        let encoded = String(data: WearableMapper.jsonData(for: record, salt: salt) ?? Data(), encoding: .utf8) ?? ""
        XCTAssertFalse(encoded.contains("rmssd"))
    }

    func testSpo2FractionBecomesPercent() {
        XCTAssertEqual(WearableMapper.spo2Percent(0.97), 97)
        XCTAssertEqual(WearableMapper.spo2Percent(97), 97)
    }

    func testDeviceNameIsHashedAndNotStored() {
        var source = watchSource()
        source.deviceLabel = "Ada Watch"
        let src = WearableMapper.srcObject(source, salt: salt)
        let encoded = String(data: try! JSONSerialization.data(withJSONObject: src), encoding: .utf8) ?? ""
        XCTAssertFalse(encoded.contains("Ada Watch"))
        XCTAssertFalse(src.keys.contains("name"))
        XCTAssertFalse(src.keys.contains("device_name"))
        XCTAssertFalse(src.keys.contains("source_name"))
        let hash = src["device_label_hash"] as? String
        XCTAssertEqual(hash?.count, 64)
        XCTAssertNotEqual(hash, WearableKind.deviceLabelHash(label: "other", salt: salt))
    }

    func testP0FixturesMapToExpectedPayloads() throws {
        for name in ["apple_watch.json", "xiaomi_mi_fitness.json", "zepp_life.json"] {
            let url = fixtureURL(name)
            let data = try Data(contentsOf: url)
            let root = try JSONSerialization.jsonObject(with: data) as? [String: Any]
            let samples = root?["samples"] as? [[String: Any]] ?? []
            XCTAssertFalse(samples.isEmpty, name)
            for sample in samples {
                let hk = sample["hk"] as? [String: Any] ?? [:]
                let observation = sample["observation"] as? [String: Any] ?? [:]
                let expectedPayload = observation["payload"] as? [String: Any] ?? [:]
                let record = record(from: hk, expectedPayload: expectedPayload)
                let mapped = WearableMapper.observationObject(for: record, salt: salt)
                XCTAssertEqual(mapped["data_type"] as? String, observation["data_type"] as? String, name)
                XCTAssertEqual(intValue(mapped["timestamp"]), intValue(observation["timestamp"]), name)
                XCTAssertTrue(
                    jsonEqual(mapped["payload"] ?? [:], stripNulls(expectedPayload)),
                    "\(name) \(hk["identifier"] ?? "")"
                )
                let blob = String(data: WearableMapper.jsonData(for: record, salt: salt) ?? Data(), encoding: .utf8) ?? ""
                XCTAssertFalse(blob.contains("\"name\""))
            }
        }
    }

    private func record(from hk: [String: Any], expectedPayload: [String: Any]) -> HealthRecord {
        let src = expectedPayload["src"] as? [String: Any] ?? [:]
        let raw = hk["value"]
        return HealthRecord(
            uuid: hk["uuid"] as? String ?? "",
            identifier: hk["identifier"] as? String ?? "",
            start: intValue(hk["start"]),
            end: intValue(hk["end"]),
            value: raw as? String == nil ? doubleValue(raw) : nil,
            categoryValue: nil,
            categoryName: raw as? String,
            activityType: nil,
            kcal: nil,
            distanceMeters: nil,
            source: SourceFacts(
                bundleId: src["bundle_id"] as? String ?? "",
                appName: src["app_name"] as? String,
                deviceModel: src["device_model"] as? String,
                manufacturer: src["device_manufacturer"] as? String,
                hardwareVersion: src["hardware_version"] as? String,
                productType: src["product_type"] as? String,
                osVersion: src["os_version"] as? String,
                deviceLabel: nil,
                wasUserEntered: src["was_user_entered"] as? Bool ?? false
            )
        )
    }

    private func watchSource() -> SourceFacts {
        SourceFacts(
            bundleId: "com.apple.health",
            appName: "Health",
            deviceModel: "Watch",
            manufacturer: "Apple Inc.",
            hardwareVersion: "Watch7,1",
            productType: "Watch7,1",
            osVersion: "11.0",
            deviceLabel: nil,
            wasUserEntered: false
        )
    }

    private func fixtureURL(_ name: String) -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("crates/bio-spec/tests/fixtures/wearables/\(name)")
    }
}

final class SyncPlannerTests: XCTestCase {
    func testFirstPageIsNewestNotOldest() {
        let now: Int64 = 1_800_000_000
        let samples = [
            DatedSample(id: "old", start: now - 400 * 86_400, end: now - 400 * 86_400),
            DatedSample(id: "yesterday", start: now - 86_400, end: now - 86_400),
            DatedSample(id: "today", start: now - 60, end: now - 60),
        ]
        let page = SyncPlanner.selectDescending(
            samples: samples,
            start: now - SyncPlanner.recentSeconds,
            endExclusive: now + 1,
            limit: 2
        )
        XCTAssertEqual(page.map(\.id), ["today", "yesterday"])
    }

    func testLateWriteInsideCutoffIsOnTheForwardChannel() {
        let install: Int64 = 1_800_000_000
        let cutoff = SyncPlanner.forwardCutoff(installTime: install)
        let lateStart = install - 2 * 86_400
        XCTAssertTrue(SyncPlanner.includedInForward(sampleStart: lateStart, cutoff: cutoff))
        XCTAssertFalse(SyncPlanner.includedInForward(sampleStart: install - 40 * 86_400, cutoff: cutoff))
    }

    func testBackfillAdvancesFromRecentIntoHistory() {
        let now: Int64 = 1_800_000_000
        var cursor = TypeCursor.fresh(now: now)
        SyncPlanner.advance(cursor: &cursor, now: now, historyDays: 365, returnedEnds: [now - 10], pageLimit: 500)
        XCTAssertEqual(cursor.phase, "history")
        XCTAssertEqual(cursor.cursorEnd, now - SyncPlanner.recentSeconds)
    }

    func testPriorityPutsHrvBeforeSteps() {
        let hrv = HealthTypeRegistry.syncPriority.firstIndex(of: HealthTypeRegistry.hrvSDNN) ?? 99
        let steps = HealthTypeRegistry.syncPriority.firstIndex(of: HealthTypeRegistry.stepCount) ?? 0
        XCTAssertLessThan(hrv, steps)
    }

    func testBatchesRespectCountAndBytes() {
        let small = Data("{\"a\":1}".utf8)
        let split = IngestBatches.split(Array(repeating: small, count: 3), maxCount: 2, maxBytes: 1_048_576)
        XCTAssertEqual(split.map(\.count), [2, 1])
        let fat = Data(repeating: 0x61, count: 40)
        let bySize = IngestBatches.split([fat, fat, fat], maxCount: 500, maxBytes: 50)
        XCTAssertEqual(bySize.count, 3)
    }

    func testObserverCompletionRunsWhenWorkThrows() async {
        var called = false
        do {
            try await ObserverFinish.callAlways(completion: { called = true }) {
                throw NSError(domain: "test", code: 1)
            }
            XCTFail("expected throw")
        } catch {
            XCTAssertTrue(called)
        }
    }

    func testSyncStateIsOneFile() throws {
        let dir = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        let url = SyncStateStore.fileURL(in: dir)
        var state = SyncState.fresh(now: 10)
        state.types["HKQuantityTypeIdentifierHeartRate"] = TypeCursor(phase: "recent", cursorEnd: 9, forwardAnchor: "abc")
        SyncStateStore.save(state, to: url)
        let loaded = SyncStateStore.load(from: url, now: 99)
        XCTAssertEqual(loaded, state)
        try? FileManager.default.removeItem(at: dir)
    }

    func testCompanionStatusHasNoHealthFields() {
        let ids = [HealthTypeRegistry.hrvSDNN, HealthTypeRegistry.stepCount]
        let marks = [HealthTypeRegistry.hrvSDNN: TypeAvailability.ok]
        XCTAssertEqual(
            CompanionStatusBoard.merge(previous: TypeAvailability.ok, next: TypeAvailability.noData),
            TypeAvailability.ok
        )
        XCTAssertEqual(
            CompanionStatusBoard.statusText(TypeAvailability.noPermission),
            "нет разрешения?"
        )
        XCTAssertEqual(CompanionStatusBoard.statusText(TypeAvailability.noData), "нет данных")
        let payload = CompanionStatusBoard.payload(
            identifiers: ids,
            marks: marks,
            cursors: [HealthTypeRegistry.hrvSDNN: "recent", HealthTypeRegistry.stepCount: "recent"],
            pending: 3
        )
        XCTAssertEqual(payload["phase"] as? String, "recent")
        XCTAssertEqual(payload["types_ok"] as? Int, 1)
        XCTAssertEqual(payload["types_total"] as? Int, 2)
        XCTAssertEqual(payload["pending"] as? Int, 3)
        let keys = Set(payload.keys)
        XCTAssertEqual(keys, ["pending", "phase", "types_empty", "types_ok", "types_total"])
        XCTAssertEqual(CompanionStatusBoard.phase(cursors: ["a": "done", "b": "done"]), "done")
    }
}

private func intValue(_ value: Any?) -> Int64 {
    if let number = value as? NSNumber {
        return number.int64Value
    }
    return 0
}

private func doubleValue(_ value: Any?) -> Double? {
    (value as? NSNumber)?.doubleValue
}

private func stripNulls(_ value: Any) -> Any {
    if let dict = value as? [String: Any] {
        var copy: [String: Any] = [:]
        for (key, item) in dict where !(item is NSNull) {
            copy[key] = stripNulls(item)
        }
        return copy
    }
    if let list = value as? [Any] {
        return list.map(stripNulls)
    }
    return value
}

private func jsonEqual(_ lhs: Any, _ rhs: Any) -> Bool {
    switch (lhs, rhs) {
    case let (l as [String: Any], r as [String: Any]):
        guard Set(l.keys) == Set(r.keys) else { return false }
        return l.keys.allSatisfy { jsonEqual(l[$0] ?? 0, r[$0] ?? 1) }
    case let (l as [Any], r as [Any]):
        guard l.count == r.count else { return false }
        return zip(l, r).allSatisfy(jsonEqual)
    case let (l as NSNumber, r as NSNumber):
        if CFGetTypeID(l) == CFBooleanGetTypeID() || CFGetTypeID(r) == CFBooleanGetTypeID() {
            return l.boolValue == r.boolValue
        }
        return abs(l.doubleValue - r.doubleValue) < 0.000_1
    case let (l as String, r as String):
        return l == r
    default:
        return false
    }
}
