// mixed-exports.ts - Testing mixed named and default exports
export const API_URL = "https://api.example.com";
export const MAX_RETRIES = 3;

export function fetchData(endpoint: string): string {
  return `Fetching from ${API_URL}/${endpoint}`;
}

export class DataProcessor {
  name: string;

  constructor(name: string) {
    this.name = name;
  }

  process(data: string): string {
    return `${this.name} processed: ${data}`;
  }
}

// Default export
export default {
  name: "MixedExportsModule",
  version: "2.0.0",
  init() {
    return "Module initialized";
  },
};
