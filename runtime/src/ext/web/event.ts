// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-unused-vars
class Event {
  SymbolToStringTag: string = "Event";
  stopPropagationFlag: boolean = false;
  #stopImmediatePropagationFlag: boolean = false;
  #inPassiveListener: boolean = false;
  dispatched: boolean = false;
  attributes: {
    type: string;
    bubbles: boolean;
    cancelable: boolean;
    composed: boolean;
    currentTarget: null;
    eventPhase: number;
    target: null;
    timeStamp: number;
  };
  canceledFlag: boolean = false;
  isTrusted: boolean = false;
  path: string[] = [];
  // deno-lint-ignore no-explicit-any
  constructor(type: string, eventInitDict: any = { __proto__: null }) {
    this.attributes = {
      type,
      bubbles: !!eventInitDict.bubbles,
      cancelable: !!eventInitDict.cancelable,
      composed: !!eventInitDict.composed,
      currentTarget: null,
      eventPhase: Event.NONE,
      target: null,
      timeStamp: 0,
    };
  }

  get type() {
    return this.attributes.type;
  }

  // TODO: Null is not returned
  get target() {
    return this.attributes.target;
  }

  get srcElement() {
    return null;
  }

  set srcElement(_) {
    // this member is deprecated
  }

  // TODO: Null is not returned
  get currentTarget() {
    return this.attributes.currentTarget;
  }

  get NONE() {
    return Event.NONE;
  }

  get CAPTURING_PHASE() {
    return Event.CAPTURING_PHASE;
  }

  get AT_TARGET() {
    return Event.AT_TARGET;
  }

  get BUBBLING_PHASE() {
    return Event.BUBBLING_PHASE;
  }

  static get NONE() {
    return 0;
  }

  static get CAPTURING_PHASE() {
    return 1;
  }

  static get AT_TARGET() {
    return 2;
  }

  static get BUBBLING_PHASE() {
    return 3;
  }

  get eventPhase() {
    return this.attributes.eventPhase;
  }

  stopPropagation() {
    this.stopPropagationFlag = true;
  }

  /** @deprecated */
  get cancelBubble() {
    return this.stopPropagationFlag;
  }

  set cancelBubble(value) {
    // TODO
    // this.stopPropagationFlag = webidl.converters.boolean(value);
  }

  stopImmediatePropagation() {
    this.stopPropagationFlag = true;
    this.#stopImmediatePropagationFlag = true;
  }

  get bubbles() {
    return this.attributes.bubbles;
  }

  get cancelable() {
    return this.attributes.cancelable;
  }

  get returnValue() {
    return !this.canceledFlag;
  }

  set returnValue(value) {
    // if (!webidl.converters.boolean(value)) {
    this.canceledFlag = true;
    // }
  }

  preventDefault() {
    // if (this.attributes.cancelable && !this.inPassiveListener) {
    this.canceledFlag = true;
    // }
  }

  get defaultPrevented() {
    return this.canceledFlag;
  }

  get composed() {
    return this.attributes.composed;
  }

  get initialized() {
    return true;
  }

  get timeStamp() {
    return this.attributes.timeStamp;
  }
}
