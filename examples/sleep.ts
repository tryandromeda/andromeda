[1, 2, 3, 4].forEach(async (i) => {
  await Andromeda.sleep(i * 1000);
  console.log(`${i}s`);
});
