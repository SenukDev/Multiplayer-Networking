import init, { WorldWrapper } from "./pkg/wt_client.js";

let world;

let isMouseDown = false;
let mouseX = 0;
let mouseY = 0;

let currentTransport = null;
let streamNumber = 1;
let currentTransportDatagramWriter = null;

// Timer
class Timer {
    constructor(callback, timeInterval) {
        this.callback = callback;
        this.timeInterval = timeInterval;
    }

    start() {
        this.expected = Date.now() + this.timeInterval;
        this.timeout = setTimeout(this.round.bind(this), this.timeInterval);
    }

    stop() {
        clearTimeout(this.timeout);
    }

    round() {
        const drift = Date.now() - this.expected;
        this.callback();
        this.expected += this.timeInterval;
        this.timeout = setTimeout(this.round.bind(this), this.timeInterval - drift);
    }
}

// Initialize WASM and ECS loop
async function run_game_loop() {
    await init();
    try {
        world = new WorldWrapper();
        window.world = world;
    } catch (err) {
        console.error("Failed to create WorldWrapper:", err);
        return;
    }

    const timer = new Timer(() => {
        try {
            if (isMouseDown) {
                //console.log("Click Hold");
                //world.input_click_hold(mouseX, mouseY);
            }
            else {
                //console.log("Click Released");
                //world.input_click_released();
            }
            world.update();
        } catch (err) {
            console.error("Error in world.update():", err);
        }
    }, 1000 / 30);

    timer.start();
}

async function connect() {

    try {
      currentTransport = new WebTransport("https://us.playdodgeball.dev:8443/");
      console.log('Initiating connection...');
    } catch (e) {
      console.log('Failed to create connection object. ' + e, 'error');
      return;
    }

    try {
      await currentTransport.ready;
      console.log('Connection ready.');

      // Required: Send a unidirectional stream to complete CONNECT
      const stream = await currentTransport.createUnidirectionalStream();
      await stream.getWriter().close();

      currentTransport.closed
        .then(() => console.log('Connection closed normally.'))
        .catch((e) => console.log('Connection closed abruptly: ' + e, 'error'));

      currentTransportDatagramWriter = currentTransport.datagrams.writable.getWriter();
      console.log('Datagram writer ready.');

      // Start receiving
      readDatagrams(currentTransport);
      acceptUnidirectionalStreams(currentTransport);

    } catch (e) {
      console.log('Connection failed. ' + e, 'error');
    }
}

// Reads incoming datagrams
async function readDatagrams(transport) {
    try {
        const reader = transport.datagrams.readable.getReader();
        console.log('Datagram reader ready.');
        
        while (true) {
        const { value, done } = await reader.read();
        if (done) {
            console.log('Done reading datagrams.');
            break;
        }
        world.receive_message(value);
        }
    } catch (e) {
        console.log('Error while reading datagrams: ' + e, 'error');
    }
}

async function acceptUnidirectionalStreams(transport) {
    const reader = transport.incomingUnidirectionalStreams.getReader();

    try {
        while (true) {
            const { value: stream, done } = await reader.read();
            if (done) {
                console.log('Done accepting unidirectional streams.');
                break;
            }

            const number = streamNumber++;
            console.log('📥 Incoming unidirectional stream #' + number);

            // Read binary chunks directly from the stream
            const binaryReader = stream.getReader();
            const chunks = [];

            try {
                while (true) {
                    const { value, done } = await binaryReader.read();
                    if (done) {
                        console.log('Stream #' + number + ' closed.');
                        break;
                    }

                    chunks.push(value);
                }

                // Combine all chunks into a single Uint8Array
                const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
                const fullMessage = new Uint8Array(totalLength);
                let offset = 0;
                for (const chunk of chunks) {
                    fullMessage.set(chunk, offset);
                    offset += chunk.length;
                }

                world.receive_message(fullMessage);

            } catch (e) {
                console.log(`Error on stream #${number}: ${e}`);
            }
        }
    } catch (e) {
        console.log('Error while accepting streams: ' + e);
    }
}



async function send_input_click_pressed(mouseX, mouseY) {
    try {
        let data = world.input_click_pressed(mouseX, mouseY);
        await currentTransportDatagramWriter.write(data);
        console.log("Sent Message");
    } catch (e) {
        addToEventLog('Error while sending data: ' + e, 'error');
    }
}

connect();
run_game_loop();

const canvas = document.getElementById("my_canvas");

canvas.addEventListener("mousedown", async (event) => {
    const rect = canvas.getBoundingClientRect();
    mouseX = event.clientX - rect.left;
    mouseY = event.clientY - rect.top;

    if (world && isMouseDown == false) {
        send_input_click_pressed(mouseX, mouseY);
        console.log("Click Pressed");
    }

    isMouseDown = true;
});

window.addEventListener("mouseup", () => {
    isMouseDown = false;
});

canvas.addEventListener("mousemove", (event) => {
    const rect = canvas.getBoundingClientRect();
    mouseX = event.clientX - rect.left;
    mouseY = event.clientY - rect.top;
});

window.addEventListener("blur", () => {
    isMouseDown = false;
});

document.addEventListener("mouseleave", () => {
    isMouseDown = false;
});