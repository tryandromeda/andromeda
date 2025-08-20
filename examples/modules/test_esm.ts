// Test ES modules functionality
export function greet(name: string): string {
    return `Hello, ${name}!`;
}

export const VERSION: string = "1.0.0";

export default function main(): void {
    console.log("ES Module default export");
}