const foo = async () => {
  try {
    const res = await fetch("https://developer.mozilla.org");
    console.log(res);
  } catch (e) {
    console.error(e);
  }
};

foo();
