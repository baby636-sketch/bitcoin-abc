const { MongoClient } = require('mongodb');
const log = require('./log');
const config = require('./config');

module.exports = {
    initializeDb: async function () {
        const client = new MongoClient(config.database.connectionUrl);
        // Use connect method to connect to the server
        await client.connect();
        log('Connected successfully to MongoDB server');
        const db = client.db(config.database.name);
        // Enforce unique aliases
        db.collection(config.database.collections.validAliases).createIndex(
            {
                alias: 1,
            },
            {
                unique: true,
            },
        );
        log(`Configured connection to database ${config.database.name}`);
        return db;
    },
};
