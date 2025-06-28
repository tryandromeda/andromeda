const foo = async () => {
  const res = await fetch("https://developer.mozilla.org");
  console.log(res);
};

foo();
