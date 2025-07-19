async function connectWebTransport() {
    const url = 'https://127.0.0.1:4444'; // or your WebTransport URL
    try {
        const transport = new WebTransport(url, {
            allowPooling: false,
        });

        await transport.ready;
        console.log('Connected to server');

        // Send a datagram
        const writer = transport.datagrams.writable.getWriter();
        const encoder = new TextEncoder();
        await writer.write(encoder.encode("Hello from client!"));
        writer.releaseLock();

        // Listen for incoming datagrams
        const reader = transport.datagrams.readable.getReader();
        while (true) {
            const { value, done } = await reader.read();
            if (done) break;
            const text = new TextDecoder().decode(value);
            console.log("Received from server:", text);
        }

    } catch (err) {
        console.error('WebTransport connection failed:', err);
    }
}

connectWebTransport();