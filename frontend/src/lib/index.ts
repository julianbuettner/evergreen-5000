export interface PlantConfig {
	name: string;
	amountMl: number;
}

export interface LastSeenInfo {
	lastSeenTimestamp: number;
	lastBatteryPercentage: number;
	lastWateringDate: string;
}
