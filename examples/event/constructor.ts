// Create a new event instance
const myEvent = new Event("customEvent", {
  bubbles: true,
  cancelable: true,
  composed: false,
});

console.log("Event Type:", myEvent.type);
console.log("Bubbles:", myEvent.bubbles);
console.log("Cancelable:", myEvent.cancelable);
console.log("Composed:", myEvent.composed);
console.log("Event Phase:", myEvent.eventPhase);
console.log("Before stopPropagation:", myEvent.cancelBubble);
myEvent.stopPropagation();
console.log("After stopPropagation:", myEvent.cancelBubble);
console.log("Before preventDefault:", myEvent.defaultPrevented);
myEvent.preventDefault();
console.log("After preventDefault:", myEvent.defaultPrevented);
console.log("Target:", myEvent.target);
console.log("Current Target:", myEvent.currentTarget);
console.log("NONE:", myEvent.NONE);
console.log("CAPTURING_PHASE:", myEvent.CAPTURING_PHASE);
console.log("AT_TARGET:", myEvent.AT_TARGET);
console.log("BUBBLING_PHASE:", myEvent.BUBBLING_PHASE);
console.log("Return Value (before change):", myEvent.returnValue);
myEvent.returnValue = false;
console.log("Return Value (after change):", myEvent.returnValue);
console.log("Time Stamp:", myEvent.timeStamp);
