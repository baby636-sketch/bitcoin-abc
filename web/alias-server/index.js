const config = require('./config');
const { initializeWebsocket, parseWebsocketMessage } = require('./websocket');
const { initializeDb } = require('./db');
const log = require('./log');
const express = require('express');

async function main() {
    // Initialize db connection
    const db = await initializeDb();

    // Initialize websocket connection
    const aliasWebsocket = await initializeWebsocket(db);
    if (aliasWebsocket && aliasWebsocket._subs && aliasWebsocket._subs[0]) {
        const subscribedHash160 = aliasWebsocket._subs[0].scriptPayload;
        log(`Websocket subscribed to ${subscribedHash160}`);
    }

    // Run parseWebsocketMessage on startup mocking a block found
    parseWebsocketMessage(db);

    // Set up your API endpoints
    const app = express();
    app.use(express.json());

    app.get('/aliases', async function (req, res) {
        log(`API request received, processing...`);
        let aliases;
        try {
            aliases = await db
                .collection(config.database.collections.validAliases)
                .find()
                .project({ _id: 0, txid: 0, blockheight: 0 })
                .toArray();
            return res.status(200).json(aliases);
        } catch (error) {
            return res.status(500).json({ error });
        }
    });

    app.listen(config.express.port);
}

main();
