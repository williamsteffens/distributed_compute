const output = document.querySelector<HTMLPreElement>("#output")!;
const startButton = document.querySelector<HTMLButtonElement>("#start")!;

const worker = new Worker(
    new URL("./worker.ts", import.meta.url),
    {
        type: "module",
    },
);

worker.onmessage = (event) => {
    console.log("worker:", event.data);

    output.textContent += JSON.stringify(event.data, null, 2) + "\n";
};

worker.onerror = (event) => {
    output.textContent += `Worker error: ${event.message}\n`;
};

startButton.onclick = () => {
    worker.postMessage({
        type: "init",
    });
};
