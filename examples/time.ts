let i = 0;

setTimeout(() => {
  console.log(`[timeout]: 700ms`);
}, 700);

clearTimeout(setTimeout(() => {
  console.log("I'll never run :)");
}, 1000));

const id = setInterval(() => {
  console.log(`[interval]: ${i}s`);
  i += 1;
  if (i == 5) {
    clearInterval(id);
  }
}, 1000);
