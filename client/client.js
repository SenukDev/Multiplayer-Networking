let currentTransport = null;
let streamNumber = 1;
let currentTransportDatagramWriter = null;

// Connect button handler
async function connect() {
  const url = document.getElementById('url').value;

  try {
    currentTransport = new WebTransport(url);
    addToEventLog('🔌 Initiating connection...');
  } catch (e) {
    addToEventLog('❌ Failed to create connection object. ' + e, 'error');
    return;
  }

  try {
    await currentTransport.ready;
    addToEventLog('✅ Connection ready.');

    // Required: Send a unidirectional stream to complete CONNECT
    const stream = await currentTransport.createUnidirectionalStream();
    await stream.getWriter().close();

    currentTransport.closed
      .then(() => addToEventLog('🔒 Connection closed normally.'))
      .catch((e) => addToEventLog('❗ Connection closed abruptly: ' + e, 'error'));

    currentTransportDatagramWriter = currentTransport.datagrams.writable.getWriter();
    addToEventLog('📤 Datagram writer ready.');

    // Start receiving
    readDatagrams(currentTransport);
    acceptUnidirectionalStreams(currentTransport);

    document.forms.sending.elements.send.disabled = false;
    document.getElementById('connect').disabled = true;

  } catch (e) {
    addToEventLog('❌ Connection failed. ' + e, 'error');
  }
}

// Send data handler
async function sendData() {
  const form = document.forms.sending.elements;
  const encoder = new TextEncoder();
  const rawData = form.data.value;
  const data = encoder.encode(rawData);

  try {
    switch (form.sendtype.value) {
      case 'datagram':
        await currentTransportDatagramWriter.write(data);
        addToEventLog('📨 Sent datagram: ' + rawData);
        break;

      case 'unidi': {
        const stream = await currentTransport.createUnidirectionalStream();
        const writer = stream.getWriter();
        await writer.write(data);
        await writer.close();
        addToEventLog('📨 Sent unidirectional stream with data: ' + rawData);
        break;
      }

      case 'bidi': {
        const stream = await currentTransport.createBidirectionalStream();
        const number = streamNumber++;
        readFromIncomingStream(stream, number);

        const writer = stream.writable.getWriter();
        await writer.write(data);
        await writer.close();
        addToEventLog('📨 Sent bidirectional stream #' + number + ' with data: ' + rawData);
        break;
      }
    }
  } catch (e) {
    addToEventLog('❌ Error while sending data: ' + e, 'error');
  }
}

// Reads incoming datagrams
async function readDatagrams(transport) {
  try {
    const reader = transport.datagrams.readable.getReader();
    addToEventLog('📥 Datagram reader ready.');

    const decoder = new TextDecoder('utf-8');
    while (true) {
      const { value, done } = await reader.read();
      if (done) {
        addToEventLog('✅ Done reading datagrams.');
        break;
      }
      const data = decoder.decode(value);
      addToEventLog('📥 Datagram received: ' + data);
    }
  } catch (e) {
    addToEventLog('❌ Error while reading datagrams: ' + e, 'error');
  }
}

// Accept unidirectional streams
async function acceptUnidirectionalStreams(transport) {
  const reader = transport.incomingUnidirectionalStreams.getReader();

  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) {
        addToEventLog('✅ Done accepting unidirectional streams.');
        break;
      }
      const stream = value;
      const number = streamNumber++;
      addToEventLog('📥 Incoming unidirectional stream #' + number);
      readFromIncomingStream(stream, number);
    }
  } catch (e) {
    addToEventLog('❌ Error while accepting streams: ' + e, 'error');
  }
}

// Reads from an incoming stream
async function readFromIncomingStream(stream, number) {
  const decoder = new TextDecoderStream();
  const reader = stream.pipeThrough(decoder).getReader();

  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) {
        addToEventLog('✅ Stream #' + number + ' closed.');
        break;
      }
      addToEventLog(`📥 Received on stream #${number}: ${value}`);
    }
  } catch (e) {
    addToEventLog(`❌ Error on stream #${number}: ${e}`, 'error');
  }
}

// Adds a message to the event log
function addToEventLog(text, severity = 'info') {
  const log = document.getElementById('event-log');
  const mostRecentEntry = log.lastElementChild;
  const entry = document.createElement('li');
  entry.innerText = text;
  entry.className = 'log-' + severity;
  log.appendChild(entry);

  if (mostRecentEntry &&
      mostRecentEntry.getBoundingClientRect().top < log.getBoundingClientRect().bottom) {
    entry.scrollIntoView();
  }
}
