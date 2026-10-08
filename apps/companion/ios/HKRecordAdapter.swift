import Foundation
import HealthKit

enum HKRecordAdapter {
    static func quantity(_ sample: HKQuantitySample, identifier: String) -> HealthRecord {
        HealthRecord(
            uuid: sample.uuid.uuidString,
            identifier: identifier,
            start: Int64(sample.startDate.timeIntervalSince1970),
            end: Int64(sample.endDate.timeIntervalSince1970),
            value: quantityValue(sample, identifier: identifier),
            categoryValue: nil,
            categoryName: nil,
            activityType: nil,
            kcal: nil,
            distanceMeters: nil,
            source: facts(from: sample)
        )
    }

    static func category(_ sample: HKCategorySample, identifier: String) -> HealthRecord {
        HealthRecord(
            uuid: sample.uuid.uuidString,
            identifier: identifier,
            start: Int64(sample.startDate.timeIntervalSince1970),
            end: Int64(sample.endDate.timeIntervalSince1970),
            value: nil,
            categoryValue: sample.value,
            categoryName: nil,
            activityType: nil,
            kcal: nil,
            distanceMeters: nil,
            source: facts(from: sample)
        )
    }

    static func workout(_ workout: HKWorkout) -> HealthRecord {
        let kcal = workout.totalEnergyBurned?.doubleValue(for: .kilocalorie())
        let meters = workout.totalDistance?.doubleValue(for: .meter())
        return HealthRecord(
            uuid: workout.uuid.uuidString,
            identifier: HealthTypeRegistry.workout,
            start: Int64(workout.startDate.timeIntervalSince1970),
            end: Int64(workout.endDate.timeIntervalSince1970),
            value: nil,
            categoryValue: nil,
            categoryName: nil,
            activityType: "\(workout.workoutActivityType.rawValue)",
            kcal: kcal,
            distanceMeters: meters,
            source: facts(from: workout)
        )
    }

    static func quantityValue(_ sample: HKQuantitySample, identifier: String) -> Double {
        switch identifier {
        case HealthTypeRegistry.heartRate,
             HealthTypeRegistry.restingHeartRate,
             HealthTypeRegistry.walkingHeartRate,
             HealthTypeRegistry.respiratoryRate:
            return sample.quantity.doubleValue(for: HKUnit.count().unitDivided(by: .minute()))
        case HealthTypeRegistry.hrvSDNN:
            return sample.quantity.doubleValue(for: HKUnit.secondUnit(with: .milli))
        case HealthTypeRegistry.oxygenSaturation:
            return sample.quantity.doubleValue(for: .percent())
        case HealthTypeRegistry.wristTemperature:
            return sample.quantity.doubleValue(for: .degreeCelsius())
        case HealthTypeRegistry.vo2Max:
            let ml = HKUnit.literUnit(with: .milli)
            let kg = HKUnit.gramUnit(with: .kilo)
            return sample.quantity.doubleValue(for: ml.unitDivided(by: kg.unitMultiplied(by: .minute())))
        case HealthTypeRegistry.stepCount:
            return sample.quantity.doubleValue(for: .count())
        case HealthTypeRegistry.distance:
            return sample.quantity.doubleValue(for: .meter())
        case HealthTypeRegistry.activeEnergy, HealthTypeRegistry.basalEnergy:
            return sample.quantity.doubleValue(for: .kilocalorie())
        case HealthTypeRegistry.exerciseTime, HealthTypeRegistry.standTime:
            return sample.quantity.doubleValue(for: .minute())
        default:
            return sample.quantity.doubleValue(for: .count())
        }
    }

    static func facts(from sample: HKSample) -> SourceFacts {
        let revision = sample.sourceRevision
        let bundle = revision.source.bundleIdentifier
        let entered = (sample.metadata?[HKMetadataKeyWasUserEntered] as? NSNumber)?.boolValue ?? false
        return SourceFacts(
            bundleId: bundle,
            appName: WearableKind.appName(forBundle: bundle),
            deviceModel: sample.device?.model,
            manufacturer: sample.device?.manufacturer,
            hardwareVersion: sample.device?.hardwareVersion,
            productType: revision.productType,
            osVersion: revision.version,
            deviceLabel: sample.device?.name,
            wasUserEntered: entered
        )
    }
}
