interface Job {
  data: SharedArrayBuffer;
  results: SharedArrayBuffer;
  index: number;
  start: number;
  end: number;
}

self.onmessage = (event: MessageEvent) => {
  const job = event.data as Job;
  const data = new Int32Array(job.data);
  const results = new Int32Array(job.results);

  let sum = 0;
  for (let i = job.start; i < job.end; i++) sum += data[i];

  Atomics.store(results, job.index, sum);
  self.postMessage("done");
};
