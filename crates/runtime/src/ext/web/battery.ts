// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

const _charging = Symbol("[[charging]]");
const _chargingTime = Symbol("[[chargingTime]]");
const _dischargingTime = Symbol("[[dischargingTime]]");
const _level = Symbol("[[level]]");

interface EventListenerOptions {
  capture?: boolean;
}

interface AddEventListenerOptions extends EventListenerOptions {
  once?: boolean;
  passive?: boolean;
  signal?: AbortSignal;
}

interface EventListenerObject {
  handleEvent(event: Event): void;
}

type EventListenerOrEventListenerObject =
  | ((event: Event) => void)
  | EventListenerObject;

/**
 * The BatteryManager interface provides information about the system's battery charge level.
 */
// TODO: Implement battery event handling with EventTarget inheritance.
class BatteryManager {
  [_charging]: boolean = false;
  [_chargingTime]: number = Infinity;
  [_dischargingTime]: number = Infinity;
  [_level]: number = 1.0;
  #eventListeners: Record<string, ((event: Event) => void)[]> = {};

  /**
   * Creates a new BatteryManager instance.
   * This constructor is private and should only be called by navigator.getBattery()
   */
  constructor() {
    try {
      this.#updateBatteryInfo();

      // Set up periodic updates (every 30 seconds)
      this.#startPeriodicUpdates();
    } catch (error) {
      console.error("Error in BatteryManager constructor:", error);
      // Set default values
      this[_charging] = false;
      this[_chargingTime] = Infinity;
      this[_dischargingTime] = Infinity;
      this[_level] = 1.0;
    }
  }

  /**
   * A Boolean value indicating whether the battery is currently being charged.
   * @returns {boolean} true if the battery is charging, false otherwise
   */
  get charging(): boolean {
    return this[_charging];
  }

  /**
   * A number representing the remaining time in seconds until the battery is fully charged,
   * or 0 if the battery is already fully charged, or Infinity if the battery is discharging.
   * @returns {number} Charging time in seconds
   */
  get chargingTime(): number {
    return this[_chargingTime];
  }

  /**
   * A number representing the remaining time in seconds until the battery is completely
   * discharged and the system suspends, or Infinity if the battery is charging.
   * @returns {number} Discharging time in seconds
   */
  get dischargingTime(): number {
    return this[_dischargingTime];
  }

  /**
   * A number representing the system's battery charge level scaled to a value between 0.0 and 1.0.
   * A value of 0.0 means the battery is empty and the system is about to be suspended.
   * A value of 1.0 means the battery is full.
   * @returns {number} Battery level between 0.0 and 1.0
   */
  get level(): number {
    return this[_level];
  }

  /**
   * Event handler for the chargingchange event
   */
  onchargingchange: ((this: BatteryManager, ev: Event) => unknown) | null =
    null;

  /**
   * Event handler for the chargingtimechange event
   */
  onchargingtimechange: ((this: BatteryManager, ev: Event) => unknown) | null =
    null;

  /**
   * Event handler for the dischargingtimechange event
   */
  ondischargingtimechange:
    | ((this: BatteryManager, ev: Event) => unknown)
    | null = null;

  /**
   * Event handler for the levelchange event
   */
  onlevelchange: ((this: BatteryManager, ev: Event) => unknown) | null = null;

  /**
   * Updates battery information from the native layer
   */
  #updateBatteryInfo(): void {
    try {
      const batteryInfoJson = __andromeda__.internal_battery_info();
      const batteryInfo = JSON.parse(batteryInfoJson);

      const oldCharging = this[_charging];
      const oldChargingTime = this[_chargingTime];
      const oldDischargingTime = this[_dischargingTime];
      const oldLevel = this[_level];

      this[_charging] = Boolean(batteryInfo.charging);

      const rawLevel = parseFloat(batteryInfo.level);
      this[_level] = isNaN(rawLevel) ?
        1.0 :
        Math.max(0.0, Math.min(1.0, rawLevel));

      if (this[_charging]) {
        const rawChargingTime = batteryInfo.chargingTime;
        if (rawChargingTime === null || rawChargingTime === undefined) {
          this[_chargingTime] = this[_level] >= 1.0 ? 0 : Infinity;
        } else {
          const parsedTime = parseFloat(rawChargingTime);
          this[_chargingTime] = isNaN(parsedTime) || parsedTime < 0 ?
            Infinity :
            parsedTime;
        }
        this[_dischargingTime] = Infinity;
      } else {
        this[_chargingTime] = Infinity;
        const rawDischargingTime = batteryInfo.dischargingTime;
        if (rawDischargingTime === null || rawDischargingTime === undefined) {
          this[_dischargingTime] = Infinity;
        } else {
          const parsedTime = parseFloat(rawDischargingTime);
          this[_dischargingTime] = isNaN(parsedTime) || parsedTime < 0 ?
            Infinity :
            parsedTime;
        }
      }
      if (oldCharging !== this[_charging]) {
        this.#dispatchBatteryEvent("chargingchange");
      }

      if (oldChargingTime !== this[_chargingTime]) {
        this.#dispatchBatteryEvent("chargingtimechange");
      }

      if (oldDischargingTime !== this[_dischargingTime]) {
        this.#dispatchBatteryEvent("dischargingtimechange");
      }

      if (oldLevel !== this[_level]) {
        this.#dispatchBatteryEvent("levelchange");
      }
    } catch (error) {
      console.warn("Failed to update battery information:", error);
      this[_charging] = false;
      this[_chargingTime] = Infinity;
      this[_dischargingTime] = Infinity;
      this[_level] = 1.0;
    }
  }

  /**
   * Add an event listener - implements EventTarget interface
   */
  addEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject | null,
    _options?: boolean | AddEventListenerOptions,
  ): void {
    if (!listener) return;

    const actualListener = typeof listener === "function" ?
      listener :
      listener.handleEvent.bind(listener);

    if (!this.#eventListeners[type]) {
      this.#eventListeners[type] = [];
    }

    const existingIndex = this.#eventListeners[type].findIndex(l =>
      l === actualListener
    );
    if (existingIndex === -1) {
      this.#eventListeners[type].push(actualListener);
    }
  }

  /**
   * Remove an event listener
   */
  removeEventListener(
    type: string,
    listener: EventListenerOrEventListenerObject | null,
    _options?: boolean | EventListenerOptions,
  ): void {
    if (!listener || !this.#eventListeners[type]) return;

    const actualListener = typeof listener === "function" ?
      listener :
      listener.handleEvent.bind(listener);

    const index = this.#eventListeners[type].indexOf(actualListener);
    if (index > -1) {
      this.#eventListeners[type].splice(index, 1);
    }
  }

  /**
   * Dispatch an event
   */
  dispatchEvent(event: Event): boolean {
    const listeners = this.#eventListeners[event.type];
    if (listeners) {
      for (const listener of listeners) {
        try {
          listener(event);
        } catch (error) {
          console.error(`Error in event listener for ${event.type}:`, error);
        }
      }
    }
    return true;
  }

  /**
   * Dispatches a battery-related event
   */
  #dispatchBatteryEvent(type: string): void {
    const event = new Event(type);
    this.dispatchEvent(event);

    const handlerName = `on${type}` as keyof this;
    const handler = this[handlerName];
    if (typeof handler === "function") {
      try {
        (handler as (this: BatteryManager, ev: Event) => unknown).call(
          this,
          event,
        );
      } catch (error) {
        console.error(
          `Error in battery event handler ${String(handlerName)}:`,
          error,
        );
      }
    }
  }

  /**
   * Starts periodic battery info updates
   */
  #startPeriodicUpdates(): void {
    // TODO: Implement periodic updates if needed
  }

  /**
   * Forces an immediate update of battery information
   * This method is not part of the standard API but temporarily needed until event handling is properly implemented
   */
  _updateNow(): void {
    this.#updateBatteryInfo();
  }
}

// Cache for the singleton BatteryManager instance
let batteryManagerInstance: BatteryManager | null = null;

/**
 * Returns a Promise that resolves with a BatteryManager object
 * This function implements navigator.getBattery()
 */
function getBattery(): Promise<BatteryManager> {
  return new Promise((resolve, reject) => {
    try {
      if (!batteryManagerInstance) {
        batteryManagerInstance = new BatteryManager();
      }
      resolve(batteryManagerInstance);
    } catch (error) {
      console.error("Error in getBattery:", error);
      reject(error);
    }
  });
}

(globalThis as unknown as { _getBattery: () => Promise<BatteryManager>; })
  ._getBattery = getBattery;
(globalThis as unknown as { BatteryManager: typeof BatteryManager; })
  .BatteryManager = BatteryManager;
