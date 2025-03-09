// deno-lint-ignore-file no-unused-vars
class Event {
  constructor(type, eventInitDict = { __proto__: null }) {
    // @ts-ignore - this is a hack to make the URL object work
    this.SymbolToStringTag = "Event";
    // @ts-ignore - this is a hack to make the URL object work
    this.canceledFlag = false;
    // @ts-ignore - this is a hack to make the URL object work
    this.stopPropagationFlag = false;
    // @ts-ignore - this is a hack to make the URL object work
    this._stopImmediatePropagationFlag = false;
    // @ts-ignore - this is a hack to make the URL object work
    this._inPassiveListener = false;
    // @ts-ignore - this is a hack to make the URL object work
    this.dispatched = false;
    // @ts-ignore - this is a hack to make the URL object work
    this.isTrusted = false;
    // @ts-ignore - this is a hack to make the URL object work
    this.path = [];

    // @ts-ignore - this is a hack to make the URL object work
    this.attributes = {
      type,
      // @ts-ignore - this is a hack to make the URL object work
      bubbles: !!eventInitDict.bubbles,
      // @ts-ignore - this is a hack to make the URL object work
      cancelable: !!eventInitDict.cancelable,
      // @ts-ignore - this is a hack to make the URL object work
      composed: !!eventInitDict.composed,
      currentTarget: null,
      eventPhase: Event.NONE,
      target: null,
      timeStamp: 0,
    };
  }

  get type() {
    // @ts-ignore - this is a hack to make the URL object work
    return this.attributes.type;
  }

  // TODO: Null is not returned
  get target() {
    // @ts-ignore - this is a hack to make the URL object work
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
    // @ts-ignore - this is a hack to make the URL object work
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
    // @ts-ignore - this is a hack to make the URL object work
    return this.attributes.eventPhase;
  }

  stopPropagation() {
    // @ts-ignore - this is a hack to make the URL object work
    this.stopPropagationFlag = true;
  }

  /** @deprecated */
  get cancelBubble() {
    // @ts-ignore - this is a hack to make the URL object work
    return this.stopPropagationFlag;
  }

  set cancelBubble(value) {
    // TODO
    // this.stopPropagationFlag = webidl.converters.boolean(value);
  }

  stopImmediatePropagation() {
    // @ts-ignore - this is a hack to make the URL object work

    this.stopPropagationFlag = true;
    // @ts-ignore - this is a hack to make the URL object work
    this.stopImmediatePropagationFlag = true;
  }

  get bubbles() {
    // @ts-ignore - this is a hack to make the URL object work
    return this.attributes.bubbles;
  }

  get cancelable() {
    // @ts-ignore - this is a hack to make the URL object work
    return this.attributes.cancelable;
  }

  get returnValue() {
    // @ts-ignore - this is a hack to make the URL object work
    return !this.canceledFlag;
  }

  set returnValue(value) {
    // if (!webidl.converters.boolean(value)) {
    // @ts-ignore - this is a hack to make the URL object work
    this.canceledFlag = true;
    // }
  }

  preventDefault() {
    // if (this.attributes.cancelable && !this.inPassiveListener) {
    // @ts-ignore - this is a hack to make the URL object work
    this.canceledFlag = true;
    // }
  }

  get defaultPrevented() {
    // @ts-ignore - this is a hack to make the URL object work
    return this.canceledFlag;
  }

  get composed() {
    // @ts-ignore - this is a hack to make the URL object work
    return this.attributes.composed;
  }

  get initialized() {
    return true;
  }

  get timeStamp() {
    // @ts-ignore - this is a hack to make the URL object work
    return this.attributes.timeStamp;
  }
}
