// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// deno-lint-ignore-file no-explicit-any
// Andromeda Event API Implementation
// Compliant with WHATWG HTML Living Standard
// https://html.spec.whatwg.org/dev/webappapis.html#events

// Private symbols for internal state
const _attributes = Symbol("[[attributes]]");
const _canceledFlag = Symbol("[[canceledFlag]]");
const _stopPropagationFlag = Symbol("[[stopPropagationFlag]]");
const _stopImmediatePropagationFlag = Symbol(
  "[[stopImmediatePropagationFlag]]",
);
const _inPassiveListener = Symbol("[[inPassiveListener]]");
const _dispatched = Symbol("[[dispatched]]");
const _isTrusted = Symbol("[[isTrusted]]");
const _path = Symbol("[[path]]");

// Accessors for non runtime visible data
function getDispatched(event: Event): boolean {
  return Boolean((event as any)[_dispatched]);
}

function getPath(event: Event): any[] {
  return (event as any)[_path] ?? [];
}

function getStopImmediatePropagation(event: Event): boolean {
  return Boolean((event as any)[_stopImmediatePropagationFlag]);
}

function setCurrentTarget(event: Event, value: any) {
  (event as any)[_attributes].currentTarget = value;
}

// deno-lint-ignore no-unused-vars
function setIsTrusted(event: Event, value: boolean) {
  (event as any)[_isTrusted] = value;
}

function setDispatched(event: Event, value: boolean) {
  (event as any)[_dispatched] = value;
}

function setEventPhase(event: Event, value: number) {
  (event as any)[_attributes].eventPhase = value;
}

function setInPassiveListener(event: Event, value: boolean) {
  (event as any)[_inPassiveListener] = value;
}

function setPath(event: Event, value: any[]) {
  (event as any)[_path] = value;
}

function setRelatedTarget(event: Event, value: any) {
  (event as any)[_attributes].relatedTarget = value;
}

function setTarget(event: Event, value: any) {
  (event as any)[_attributes].target = value;
}

function setStopImmediatePropagation(event: Event, value: boolean) {
  (event as any)[_stopImmediatePropagationFlag] = value;
}

class Event {
  [_attributes]: {
    type: string;
    bubbles: boolean;
    cancelable: boolean;
    composed: boolean;
    currentTarget: any;
    eventPhase: number;
    target: any;
    timeStamp: number;
    relatedTarget?: any;
  };
  [_canceledFlag]: boolean = false;
  [_stopPropagationFlag]: boolean = false;
  [_stopImmediatePropagationFlag]: boolean = false;
  [_inPassiveListener]: boolean = false;
  [_dispatched]: boolean = false;
  [_isTrusted]: boolean = false;
  [_path]: any[] = [];

  constructor(type: string, eventInitDict: EventInit = {}) {
    // Per spec: If invoked without arguments, throw TypeError
    if (type === undefined) {
      throw new TypeError(
        "Failed to construct 'Event': 1 argument required, but only 0 present.",
      );
    }

    // Per spec: Convert type to DOMString
    type = String(type);

    this[_canceledFlag] = false;
    this[_stopPropagationFlag] = false;
    this[_stopImmediatePropagationFlag] = false;
    this[_inPassiveListener] = false;
    this[_dispatched] = false;
    this[_isTrusted] = false;
    this[_path] = [];

    this[_attributes] = {
      type,
      bubbles: !!eventInitDict.bubbles,
      cancelable: !!eventInitDict.cancelable,
      composed: !!eventInitDict.composed,
      currentTarget: null,
      eventPhase: Event.NONE,
      target: null,
      timeStamp: performance.now(), // Use DOMHighResTimeStamp as per spec
    };
  }

  get type(): string {
    return this[_attributes].type;
  }

  get target(): any {
    return this[_attributes].target;
  }

  get srcElement(): any {
    return null;
  }

  set srcElement(_: any) {
    // this member is deprecated
  }

  get currentTarget(): any {
    return this[_attributes].currentTarget;
  }

  composedPath(): any[] {
    const path = this[_path];
    if (path.length === 0) {
      return [];
    }

    if (!this.currentTarget) {
      throw new Error("assertion error");
    }

    const composedPath = [
      {
        item: this.currentTarget,
        itemInShadowTree: false,
        relatedTarget: null,
        rootOfClosedTree: false,
        slotInClosedTree: false,
        target: null,
        touchTargetList: [],
      },
    ];

    let currentTargetIndex = 0;
    let currentTargetHiddenSubtreeLevel = 0;

    for (let index = path.length - 1; index >= 0; index--) {
      const { item, rootOfClosedTree, slotInClosedTree } = path[index];

      if (rootOfClosedTree) {
        currentTargetHiddenSubtreeLevel++;
      }

      if (item === this.currentTarget) {
        currentTargetIndex = index;
        break;
      }

      if (slotInClosedTree) {
        currentTargetHiddenSubtreeLevel--;
      }
    }

    let currentHiddenLevel = currentTargetHiddenSubtreeLevel;
    let maxHiddenLevel = currentTargetHiddenSubtreeLevel;

    for (let i = currentTargetIndex - 1; i >= 0; i--) {
      const { item, rootOfClosedTree, slotInClosedTree } = path[i];

      if (rootOfClosedTree) {
        currentHiddenLevel++;
      }

      if (currentHiddenLevel <= maxHiddenLevel) {
        composedPath.unshift({
          item,
          itemInShadowTree: false,
          relatedTarget: null,
          rootOfClosedTree: false,
          slotInClosedTree: false,
          target: null,
          touchTargetList: [],
        });
      }

      if (slotInClosedTree) {
        currentHiddenLevel--;

        if (currentHiddenLevel < maxHiddenLevel) {
          maxHiddenLevel = currentHiddenLevel;
        }
      }
    }

    currentHiddenLevel = currentTargetHiddenSubtreeLevel;
    maxHiddenLevel = currentTargetHiddenSubtreeLevel;

    for (let index = currentTargetIndex + 1; index < path.length; index++) {
      const { item, rootOfClosedTree, slotInClosedTree } = path[index];

      if (slotInClosedTree) {
        currentHiddenLevel++;
      }

      if (currentHiddenLevel <= maxHiddenLevel) {
        composedPath.push({
          item,
          itemInShadowTree: false,
          relatedTarget: null,
          rootOfClosedTree: false,
          slotInClosedTree: false,
          target: null,
          touchTargetList: [],
        });
      }

      if (rootOfClosedTree) {
        currentHiddenLevel--;

        if (currentHiddenLevel < maxHiddenLevel) {
          maxHiddenLevel = currentHiddenLevel;
        }
      }
    }
    return composedPath.map((p) => p.item);
  }

  get NONE(): number {
    return Event.NONE;
  }

  get CAPTURING_PHASE(): number {
    return Event.CAPTURING_PHASE;
  }

  get AT_TARGET(): number {
    return Event.AT_TARGET;
  }

  get BUBBLING_PHASE(): number {
    return Event.BUBBLING_PHASE;
  }

  static get NONE(): number {
    return 0;
  }

  static get CAPTURING_PHASE(): number {
    return 1;
  }

  static get AT_TARGET(): number {
    return 2;
  }

  static get BUBBLING_PHASE(): number {
    return 3;
  }

  get eventPhase(): number {
    return this[_attributes].eventPhase;
  }

  stopPropagation(): void {
    this[_stopPropagationFlag] = true;
  }

  get cancelBubble(): boolean {
    return this[_stopPropagationFlag];
  }

  set cancelBubble(value: boolean) {
    this[_stopPropagationFlag] = Boolean(value);
  }

  stopImmediatePropagation(): void {
    this[_stopPropagationFlag] = true;
    this[_stopImmediatePropagationFlag] = true;
  }

  get bubbles(): boolean {
    return this[_attributes].bubbles;
  }

  get cancelable(): boolean {
    return this[_attributes].cancelable;
  }

  get returnValue(): boolean {
    return !this[_canceledFlag];
  }

  set returnValue(value: boolean) {
    // deno-lint-ignore no-extra-boolean-cast
    if (!Boolean(value)) {
      this[_canceledFlag] = true;
    }
  }

  preventDefault(): void {
    if (this[_attributes].cancelable && !this[_inPassiveListener]) {
      this[_canceledFlag] = true;
    }
  }

  get defaultPrevented(): boolean {
    return this[_canceledFlag];
  }

  get composed(): boolean {
    return this[_attributes].composed;
  }

  get initialized(): boolean {
    return true;
  }

  get timeStamp(): number {
    return this[_attributes].timeStamp;
  }

  get isTrusted(): boolean {
    return this[_isTrusted];
  }
}

// Event listener types and interfaces
interface EventListenerOptions {
  capture?: boolean;
}

interface AddEventListenerOptions extends EventListenerOptions {
  once?: boolean;
  passive?: boolean;
  signal?: AbortSignal;
}

interface EventInit {
  bubbles?: boolean;
  cancelable?: boolean;
  composed?: boolean;
}

interface EventListener {
  (evt: Event): void;
}

interface EventListenerObject {
  handleEvent(evt: Event): void;
}

type EventListenerOrEventListenerObject = EventListener | EventListenerObject;

interface ListenerEntry {
  callback: EventListenerOrEventListenerObject;
  options: AddEventListenerOptions | boolean;
}

// EventTarget data storage
const eventTargetData = Symbol("eventTargetData");

interface EventTargetData {
  assignedSlot: boolean;
  hasActivationBehavior: boolean;
  host: any;
  listeners: Record<string, ListenerEntry[]>;
  mode: string;
}

// deno-lint-ignore no-unused-vars
function setEventTargetData(target: any): void {
  target[eventTargetData] = getDefaultTargetData();
}

function getDefaultTargetData(): EventTargetData {
  return {
    assignedSlot: false,
    hasActivationBehavior: false,
    host: null,
    listeners: Object.create(null),
    mode: "",
  };
}

function getListeners(target: any): Record<string, ListenerEntry[]> {
  return target?.[eventTargetData]?.listeners ?? {};
}

function normalizeEventHandlerOptions(
  options: boolean | AddEventListenerOptions | undefined,
): EventListenerOptions {
  if (typeof options === "boolean" || typeof options === "undefined") {
    return {
      capture: Boolean(options),
    };
  } else {
    return options;
  }
}

function addEventListenerOptionsConverter(
  V: boolean | AddEventListenerOptions | undefined,
  _prefix: string,
): AddEventListenerOptions {
  if (typeof V !== "object" || V === null) {
    return { capture: !!V, once: false, passive: false };
  }

  const options: AddEventListenerOptions = {
    capture: !!V.capture,
    once: !!V.once,
    passive: !!V.passive,
  };

  if (V.signal !== undefined) {
    options.signal = V.signal;
  }

  return options;
}

// DOM Logic Helper functions
const DOCUMENT_FRAGMENT_NODE = 11;

function getParent(eventTarget: any): any {
  return isNode(eventTarget) ? eventTarget.parentNode : null;
}

function getRoot(eventTarget: any): any {
  return isNode(eventTarget) ?
    eventTarget.getRootNode({ composed: true }) :
    null;
}

function isNode(eventTarget: any): boolean {
  return eventTarget?.nodeType !== undefined;
}

function isShadowRoot(nodeImpl: any): boolean {
  return Boolean(
    nodeImpl &&
      isNode(nodeImpl) &&
      nodeImpl.nodeType === DOCUMENT_FRAGMENT_NODE &&
      getHost(nodeImpl) != null,
  );
}

function getHost(target: any): any {
  return target?.[eventTargetData]?.host ?? null;
}

function getMode(target: any): string {
  return target?.[eventTargetData]?.mode ?? "";
}

function isShadowInclusiveAncestor(ancestor: any, node: any): boolean {
  while (isNode(node)) {
    if (node === ancestor) {
      return true;
    }

    if (isShadowRoot(node)) {
      node = node && getHost(node);
    } else {
      node = getParent(node);
    }
  }

  return false;
}

// Event dispatching functions
function appendToEventPath(
  eventImpl: Event,
  target: any,
  targetOverride: any,
  relatedTarget: any,
  touchTargets: any[],
  slotInClosedTree: boolean,
): void {
  const itemInShadowTree = isNode(target) && isShadowRoot(getRoot(target));
  const rootOfClosedTree = isShadowRoot(target) &&
    getMode(target) === "closed";

  getPath(eventImpl).push({
    item: target,
    itemInShadowTree,
    target: targetOverride,
    relatedTarget,
    touchTargetList: touchTargets,
    rootOfClosedTree,
    slotInClosedTree,
  });
}

function retarget(a: any, b: any): any {
  while (true) {
    if (!isNode(a)) {
      return a;
    }

    const aRoot = a.getRootNode();

    if (aRoot) {
      if (
        !isShadowRoot(aRoot) ||
        (isNode(b) && isShadowInclusiveAncestor(aRoot, b))
      ) {
        return a;
      }

      a = getHost(aRoot);
    }
  }
}

function innerInvokeEventListeners(
  eventImpl: Event,
  targetListeners: Record<string, ListenerEntry[]>,
): boolean {
  let found = false;

  const { type } = eventImpl;

  if (!targetListeners || !targetListeners[type]) {
    return found;
  }

  let handlers = targetListeners[type];
  const handlersLength = handlers.length;

  // Copy event listeners before iterating since the list can be modified during the iteration.
  if (handlersLength > 1) {
    handlers = [...targetListeners[type]];
  }

  for (let i = 0; i < handlersLength; i++) {
    const listener = handlers[i];

    let capture: boolean, once: boolean, passive: boolean;
    if (typeof listener.options === "boolean") {
      capture = listener.options;
      once = false;
      passive = false;
    } else {
      capture = !!listener.options.capture;
      once = !!listener.options.once;
      passive = !!listener.options.passive;
    }

    // Check if the event listener has been removed since the listeners has been cloned.
    if (!targetListeners[type].includes(listener)) {
      continue;
    }

    found = true;

    if (
      (eventImpl.eventPhase === Event.CAPTURING_PHASE && !capture) ||
      (eventImpl.eventPhase === Event.BUBBLING_PHASE && capture)
    ) {
      continue;
    }

    if (once) {
      const index = targetListeners[type].indexOf(listener);
      if (index !== -1) {
        targetListeners[type].splice(index, 1);
      }
    }

    if (passive) {
      setInPassiveListener(eventImpl, true);
    }
    if (typeof listener.callback === "object" && listener.callback !== null) {
      if (
        typeof (listener.callback as EventListenerObject).handleEvent ===
          "function"
      ) {
        (listener.callback as EventListenerObject).handleEvent(eventImpl);
      }
    } else if (typeof listener.callback === "function") {
      try {
        (listener.callback as EventListener).call(
          eventImpl.currentTarget,
          eventImpl,
        );
      } catch (_) {
        (listener.callback as EventListener)(eventImpl);
      }
    }

    setInPassiveListener(eventImpl, false);

    if (getStopImmediatePropagation(eventImpl)) {
      return found;
    }
  }

  return found;
}

function invokeEventListeners(tuple: any, eventImpl: Event): void {
  const path = getPath(eventImpl);
  if (path.length === 1) {
    const t = path[0];
    if (t.target) {
      setTarget(eventImpl, t.target);
    }
  } else {
    const tupleIndex = path.indexOf(tuple);
    for (let i = tupleIndex; i >= 0; i--) {
      const t = path[i];
      if (t.target) {
        setTarget(eventImpl, t.target);
        break;
      }
    }
  }

  setRelatedTarget(eventImpl, tuple.relatedTarget);

  if (eventImpl.cancelBubble) {
    return;
  }

  setCurrentTarget(eventImpl, tuple.item);

  try {
    innerInvokeEventListeners(eventImpl, getListeners(tuple.item));
  } catch (error) {
    reportException(error);
  }
}

function dispatch(
  targetImpl: any,
  eventImpl: Event,
  targetOverride?: any,
): boolean {
  let clearTargets = false;

  setDispatched(eventImpl, true);

  targetOverride = targetOverride ?? targetImpl;
  const eventRelatedTarget = (eventImpl as any).relatedTarget;
  let relatedTarget = retarget(eventRelatedTarget, targetImpl);

  if (targetImpl !== relatedTarget || targetImpl === eventRelatedTarget) {
    const touchTargets: any[] = [];

    appendToEventPath(
      eventImpl,
      targetImpl,
      targetOverride,
      relatedTarget,
      touchTargets,
      false,
    );

    const isActivationEvent = eventImpl.type === "click";
    let activationTarget = null;

    if (isActivationEvent && getHasActivationBehavior(targetImpl)) {
      activationTarget = targetImpl;
    }

    let slotInClosedTree = false;
    let parent = getParent(targetImpl);

    while (parent !== null) {
      relatedTarget = retarget(eventRelatedTarget, parent);

      if (
        isNode(parent) &&
        isShadowInclusiveAncestor(getRoot(targetImpl), parent)
      ) {
        appendToEventPath(
          eventImpl,
          parent,
          null,
          relatedTarget,
          touchTargets,
          slotInClosedTree,
        );
      } else if (parent === relatedTarget) {
        parent = null;
      } else {
        targetImpl = parent;

        if (
          isActivationEvent &&
          activationTarget === null &&
          getHasActivationBehavior(targetImpl)
        ) {
          activationTarget = targetImpl;
        }

        appendToEventPath(
          eventImpl,
          parent,
          targetImpl,
          relatedTarget,
          touchTargets,
          slotInClosedTree,
        );
      }

      if (parent !== null) {
        parent = getParent(parent);
      }

      slotInClosedTree = false;
    }

    let clearTargetsTupleIndex = -1;
    const path = getPath(eventImpl);
    for (
      let i = path.length - 1;
      i >= 0 && clearTargetsTupleIndex === -1;
      i--
    ) {
      if (path[i].target !== null) {
        clearTargetsTupleIndex = i;
      }
    }
    const clearTargetsTuple = path[clearTargetsTupleIndex];

    if (clearTargetsTuple) {
      clearTargets = (isNode(clearTargetsTuple.target) &&
        isShadowRoot(getRoot(clearTargetsTuple.target))) ||
        (isNode(clearTargetsTuple.relatedTarget) &&
          isShadowRoot(getRoot(clearTargetsTuple.relatedTarget)));
    }

    setEventPhase(eventImpl, Event.CAPTURING_PHASE);

    for (let i = path.length - 1; i >= 0; --i) {
      const tuple = path[i];

      if (tuple.target === null) {
        invokeEventListeners(tuple, eventImpl);
      }
    }

    for (let i = 0; i < path.length; i++) {
      const tuple = path[i];

      if (tuple.target !== null) {
        setEventPhase(eventImpl, Event.AT_TARGET);
      } else {
        setEventPhase(eventImpl, Event.BUBBLING_PHASE);
      }

      if (
        (eventImpl.eventPhase === Event.BUBBLING_PHASE &&
          eventImpl.bubbles) ||
        eventImpl.eventPhase === Event.AT_TARGET
      ) {
        invokeEventListeners(tuple, eventImpl);
      }
    }
  }

  setEventPhase(eventImpl, Event.NONE);
  setCurrentTarget(eventImpl, null);
  setPath(eventImpl, []);
  setDispatched(eventImpl, false);
  (eventImpl as any).cancelBubble = false;
  setStopImmediatePropagation(eventImpl, false);

  if (clearTargets) {
    setTarget(eventImpl, null);
    setRelatedTarget(eventImpl, null);
  }

  return !eventImpl.defaultPrevented;
}

function getHasActivationBehavior(target: any): boolean {
  return Boolean(target?.[eventTargetData]?.hasActivationBehavior);
}

// Error reporting function placeholder
function reportException(error: any): void {
  console.error("Event listener error:", error);
}

// EventTarget class
class EventTarget {
  [eventTargetData]: EventTargetData;

  constructor() {
    this[eventTargetData] = getDefaultTargetData();
  }
  addEventListener(
    type: string,
    callback: EventListenerOrEventListenerObject | null,
    options?: boolean | AddEventListenerOptions,
  ): void {
    // Per spec: Validate arguments
    if (type === undefined) {
      throw new TypeError(
        "Failed to execute 'addEventListener' on 'EventTarget': 1 argument required, but only 0 present.",
      );
    }
    if (callback === undefined) {
      throw new TypeError(
        "Failed to execute 'addEventListener' on 'EventTarget': 2 arguments required, but only 1 present.",
      );
    }

    // Per spec: Convert type to DOMString
    type = String(type);

    const prefix = "Failed to execute 'addEventListener' on 'EventTarget'";

    const processedOptions = addEventListenerOptionsConverter(options, prefix);

    if (callback === null) {
      return;
    }

    const { listeners } = this[eventTargetData];

    if (!listeners[type]) {
      listeners[type] = [];
    }

    const listenerList = listeners[type];
    for (let i = 0; i < listenerList.length; ++i) {
      const listener = listenerList[i];
      if (
        ((typeof listener.options === "boolean" &&
          listener.options === processedOptions.capture) ||
          (typeof listener.options === "object" &&
            listener.options.capture === processedOptions.capture)) &&
        listener.callback === callback
      ) {
        return;
      }
    }
    if (processedOptions?.signal) {
      const signal = processedOptions?.signal;
      if (signal.aborted) {
        return;
      } else {
        const removeListener = () => {
          // Remove the specific listener entry that was added
          const listenerList = this[eventTargetData].listeners[type];
          if (listenerList) {
            for (let i = 0; i < listenerList.length; i++) {
              const listener = listenerList[i];
              if (
                listener.callback === callback &&
                listener.options === processedOptions
              ) {
                listenerList.splice(i, 1);
                break;
              }
            }
          }
        };
        signal.addEventListener("abort", removeListener);
      }
    }

    listeners[type].push({ callback, options: processedOptions });
  }
  removeEventListener(
    type: string,
    callback: EventListenerOrEventListenerObject | null,
    options?: boolean | EventListenerOptions,
  ): void {
    // Per spec: Validate arguments
    if (type === undefined) {
      throw new TypeError(
        "Failed to execute 'removeEventListener' on 'EventTarget': 1 argument required, but only 0 present.",
      );
    }
    if (callback === undefined) {
      throw new TypeError(
        "Failed to execute 'removeEventListener' on 'EventTarget': 2 arguments required, but only 1 present.",
      );
    }

    // Per spec: Convert type to DOMString
    type = String(type);

    const { listeners } = this[eventTargetData];
    if (callback === null || !listeners[type]) {
      return;
    }

    const normalizedOptions = normalizeEventHandlerOptions(options);

    for (let i = 0; i < listeners[type].length; ++i) {
      const listener = listeners[type][i];
      if (
        ((typeof listener.options === "boolean" &&
          listener.options === normalizedOptions.capture) ||
          (typeof listener.options === "object" &&
            listener.options.capture === normalizedOptions.capture)) &&
        listener.callback === callback
      ) {
        listeners[type].splice(i, 1);
        break;
      }
    }
  }
  dispatchEvent(event: Event): boolean {
    // Per spec: Validate arguments
    if (event === undefined) {
      throw new TypeError(
        "Failed to execute 'dispatchEvent' on 'EventTarget': 1 argument required, but only 0 present.",
      );
    }

    // Per spec: Check if event is currently being dispatched
    if (getDispatched(event)) {
      throw new DOMException(
        "Failed to execute 'dispatchEvent' on 'EventTarget': The event is already being dispatched.",
        "InvalidStateError",
      );
    }

    // Per spec: Check if event's initialized flag is not set
    if (event.eventPhase !== Event.NONE) {
      throw new DOMException(
        "Failed to execute 'dispatchEvent' on 'EventTarget': The event's phase is not NONE.",
        "InvalidStateError",
      );
    }

    const { listeners } = this[eventTargetData];
    if (!listeners[event.type]) {
      setTarget(event, this);
      return true;
    }

    if (getDispatched(event)) {
      throw new Error(
        "Invalid event state: Event is already being dispatched",
      );
    }

    if (event.eventPhase !== Event.NONE) {
      throw new Error("Invalid event state: Event phase must be NONE");
    }

    return dispatch(this, event);
  }

  getParent(_event: Event): any {
    return null;
  }
}

class ErrorEvent extends Event {
  readonly message: string;
  readonly filename: string;
  readonly lineno: number;
  readonly colno: number;
  readonly error: any;

  constructor(
    type: string,
    eventInitDict: {
      bubbles?: boolean;
      cancelable?: boolean;
      composed?: boolean;
      message?: string;
      filename?: string;
      lineno?: number;
      colno?: number;
      error?: any;
    } = {},
  ) {
    super(type, {
      bubbles: eventInitDict.bubbles,
      cancelable: eventInitDict.cancelable,
      composed: eventInitDict.composed,
    });

    this.message = eventInitDict.message ?? "";
    this.filename = eventInitDict.filename ?? "";
    this.lineno = eventInitDict.lineno ?? 0;
    this.colno = eventInitDict.colno ?? 0;
    this.error = eventInitDict.error;
  }
}

class CloseEvent extends Event {
  readonly wasClean: boolean;
  readonly code: number;
  readonly reason: string;

  constructor(
    type: string,
    eventInitDict: {
      bubbles?: boolean;
      cancelable?: boolean;
      composed?: boolean;
      wasClean?: boolean;
      code?: number;
      reason?: string;
    } = {},
  ) {
    super(type, {
      bubbles: eventInitDict.bubbles,
      cancelable: eventInitDict.cancelable,
      composed: eventInitDict.composed,
    });

    this.wasClean = eventInitDict.wasClean ?? false;
    this.code = eventInitDict.code ?? 0;
    this.reason = eventInitDict.reason ?? "";
  }
}

class MessageEvent extends Event {
  readonly data: any;
  readonly ports: any[];
  readonly origin: string;
  readonly lastEventId: string;

  get source(): any {
    return null;
  }

  constructor(
    type: string,
    eventInitDict: {
      bubbles?: boolean;
      cancelable?: boolean;
      composed?: boolean;
      data?: any;
      ports?: any[];
      origin?: string;
      lastEventId?: string;
    } = {},
  ) {
    super(type, {
      bubbles: eventInitDict.bubbles ?? false,
      cancelable: eventInitDict.cancelable ?? false,
      composed: eventInitDict.composed ?? false,
    });

    this.data = eventInitDict.data ?? null;
    this.ports = eventInitDict.ports ?? [];
    this.origin = eventInitDict.origin ?? "";
    this.lastEventId = eventInitDict.lastEventId ?? "";
  }
}

class ProgressEvent extends Event {
  readonly lengthComputable: boolean;
  readonly loaded: number;
  readonly total: number;

  constructor(
    type: string,
    eventInitDict: {
      bubbles?: boolean;
      cancelable?: boolean;
      composed?: boolean;
      lengthComputable?: boolean;
      loaded?: number;
      total?: number;
    } = {},
  ) {
    super(type, eventInitDict);

    this.lengthComputable = eventInitDict.lengthComputable ?? false;
    this.loaded = eventInitDict.loaded ?? 0;
    this.total = eventInitDict.total ?? 0;
  }
}

class PromiseRejectionEvent extends Event {
  readonly promise: Promise<any>;
  readonly reason: any;

  constructor(
    type: string,
    eventInitDict: {
      bubbles?: boolean;
      cancelable?: boolean;
      composed?: boolean;
      promise?: Promise<any>;
      reason?: any;
    } = {},
  ) {
    super(type, {
      bubbles: eventInitDict.bubbles,
      cancelable: eventInitDict.cancelable,
      composed: eventInitDict.composed,
    });

    this.promise = eventInitDict.promise!;
    this.reason = eventInitDict.reason;
  }
}

const _eventHandlers = Symbol("eventHandlers");

function makeWrappedHandler(
  handler: EventListener,
  isSpecialErrorEventHandler = false,
): EventListener {
  function wrappedHandler(evt: Event): any {
    if (typeof (wrappedHandler as any).handler !== "function") {
      return;
    }

    if (
      isSpecialErrorEventHandler &&
      evt instanceof ErrorEvent &&
      evt.type === "error"
    ) {
      const ret = ((wrappedHandler as any).handler as any).call(
        // @ts-ignore ignore this for now
        this,
        evt.message,
        evt.filename,
        evt.lineno,
        evt.colno,
        evt.error,
      );
      if (ret === true) {
        evt.preventDefault();
      }
      return;
    }

    // @ts-ignore ignore this for now
    return ((wrappedHandler as any).handler as EventListener).call(this, evt);
  }
  (wrappedHandler as any).handler = handler;
  return wrappedHandler;
}

// deno-lint-ignore no-unused-vars
function defineEventHandler(
  emitter: any,
  name: string,
  init?: (obj: any) => void,
  isSpecialErrorEventHandler = false,
): void {
  Object.defineProperty(emitter, `on${name}`, {
    get() {
      if (!this[_eventHandlers]) {
        return null;
      }

      return this[_eventHandlers].get(name)?.handler ?? null;
    },
    set(value: any) {
      if (typeof value !== "object" && typeof value !== "function") {
        value = null;
      }

      if (!this[_eventHandlers]) {
        this[_eventHandlers] = new Map();
      }
      let handlerWrapper = this[_eventHandlers].get(name);
      if (handlerWrapper) {
        (handlerWrapper as any).handler = value;
      } else if (value !== null) {
        handlerWrapper = makeWrappedHandler(value, isSpecialErrorEventHandler);
        this.addEventListener(name, handlerWrapper);
        init?.(this);
      }
      this[_eventHandlers].set(name, handlerWrapper);
    },
    configurable: true,
    enumerable: true,
  });
}

// deno-lint-ignore prefer-const no-unused-vars
let reportExceptionStackedCalls = 0;

function reportError(error: any): void {
  reportException(error);
}

// Utility functions for external use
// deno-lint-ignore no-unused-vars
function listenerCount(target: any, type: string): number {
  return getListeners(target)?.[type]?.length ?? 0;
}

// Private symbols for AbortSignal internal state
const _aborted = Symbol("[[aborted]]");
const _abortReason = Symbol("[[abortReason]]");
const _abortAlgorithms = Symbol("[[abortAlgorithms]]");

class AbortSignal extends EventTarget {
  constructor() {
    super();

    (this as any)[_aborted] = false;
    (this as any)[_abortReason] = undefined;
    (this as any)[_abortAlgorithms] = new Set();
  }
  get aborted(): boolean {
    return (this as any)[_aborted];
  }

  get reason(): any {
    return (this as any)[_abortReason];
  }

  throwIfAborted(): void {
    if ((this as any)[_aborted]) {
      throw (this as any)[_abortReason];
    }
  }
  // Static factory methods
  static abort(reason?: any): AbortSignal {
    const signal = new AbortSignal();
    (signal as any)[_aborted] = true;
    (signal as any)[_abortReason] = reason !== undefined ?
      reason :
      new DOMException("signal is aborted without reason", "AbortError");
    return signal;
  }
  static timeout(milliseconds: number): AbortSignal {
    if (milliseconds < 0) {
      throw new RangeError("milliseconds must be non-negative");
    }

    const signal = new AbortSignal();
    if (milliseconds === 0) {
      (signal as any)[_aborted] = true;
      (signal as any)[_abortReason] = new DOMException(
        "signal timed out",
        "TimeoutError",
      );
    } else {
      const timeoutCallback = function() {
        if (!(signal as any)[_aborted]) {
          signalAbort(
            signal,
            new DOMException("signal timed out", "TimeoutError"),
          );
        }
      };
      setTimeout(timeoutCallback, milliseconds);
    }

    return signal;
  }
  static any(signals: AbortSignal[]): AbortSignal {
    const resultSignal = new AbortSignal();

    // If any signal is already aborted, return an aborted signal
    for (const signal of signals) {
      if (signal.aborted) {
        (resultSignal as any)[_aborted] = true;
        (resultSignal as any)[_abortReason] = signal.reason;
        return resultSignal;
      }
    }

    // Listen for abort on any of the signals
    for (const signal of signals) {
      signal.addEventListener("abort", () => {
        if (!resultSignal.aborted) {
          signalAbort(resultSignal, signal.reason);
        }
      });
    }

    return resultSignal;
  }
}

// AbortController implementation
class AbortController {
  #signal: AbortSignal;

  constructor() {
    this.#signal = new AbortSignal();
  }

  get signal(): AbortSignal {
    return this.#signal;
  }

  abort(reason?: any): void {
    signalAbort(
      this.#signal,
      reason !== undefined ?
        reason :
        new DOMException("signal is aborted without reason", "AbortError"),
    );
  }
}

// Internal function to signal abort
function signalAbort(signal: AbortSignal, reason: any): void {
  if ((signal as any)[_aborted]) {
    return;
  }

  (signal as any)[_aborted] = true;
  (signal as any)[_abortReason] = reason;

  // Execute abort algorithms
  const algorithms = (signal as any)[_abortAlgorithms];
  for (const algorithm of algorithms) {
    try {
      algorithm();
    } catch (error) {
      // Report the exception but continue with other algorithms
      reportError(error);
    }
  }
  algorithms.clear();

  // Fire abort event
  const event = new Event("abort");
  signal.dispatchEvent(event);
}

// @ts-ignore globalThis is not readonly
globalThis.Event = Event;
// @ts-ignore globalThis is not readonly
globalThis.EventTarget = EventTarget;
// @ts-ignore globalThis is not readonly
globalThis.ErrorEvent = ErrorEvent;
// @ts-ignore globalThis is not readonly
globalThis.CloseEvent = CloseEvent;
// @ts-ignore globalThis is not readonly
globalThis.MessageEvent = MessageEvent;
// @ts-ignore globalThis is not readonly
globalThis.ProgressEvent = ProgressEvent;
// @ts-ignore globalThis is not readonly
globalThis.PromiseRejectionEvent = PromiseRejectionEvent;
// @ts-ignore globalThis is not readonly
globalThis.AbortController = AbortController;
// @ts-ignore globalThis is not readonly
globalThis.AbortSignal = AbortSignal;
