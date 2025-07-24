import init, { WorldWrapper } from "./pkg/wt_client.js";

let world;

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
        world.receive_datagram(value);
        }
    } catch (e) {
        console.log('Error while reading datagrams: ' + e, 'error');
    }
}

async function acceptUnidirectionalStreams(transport) {
    const reader = transport.incomingUnidirectionalStreams.getReader();

    try {
        while (true) {
        const { value, done } = await reader.read();
        if (done) {
            console.log('✅ Done accepting unidirectional streams.');
            break;
        }
        const stream = value;
        const number = streamNumber++;
        console.log('📥 Incoming unidirectional stream #' + number);
        readFromIncomingStream(stream, number);
        }
    } catch (e) {
        console.log('❌ Error while accepting streams: ' + e, 'error');
    }
}

connect();
run_game_loop();
