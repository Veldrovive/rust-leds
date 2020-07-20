const Client = require("./client");
const fs = require("fs");

async function main() {
    const client = await Client.setup({
        userToken: "test-token",
        deviceName: "test-device",
        appName: "pattern-relay",
        url: "ws://108.174.195.143:8000"
    });

    const defaults = JSON.parse(fs.readFileSync("./patterns/defaults.json"));


    client.on("set_pattern", meta => {
        console.log("Setting pattern to: ", meta.toLowerCase().replace(" ", "_"));

        const path = `./patterns/${meta.toLowerCase().replace(" ", "_")}.json`;
        if (fs.existsSync(path)) {
            const patternData = JSON.parse(fs.readFileSync(path));
            const req = []
            for (let [name, pattern] of Object.entries(patternData)) {
                const patternObj = {
                    "pattern": pattern["pattern"],
                    "name": name,
                    "args": {...defaults[pattern["pattern"]], ...pattern["args"]}
                };
                req.push(patternObj);
            }
            client.send("clear_patterns", {}, () => {
                client.send("add_patterns", {patterns: req}, patterns => {
                    console.log("Patterns are: ", JSON.parse(patterns));
                })
            })
        }
    });

    client.on("clear_pattern", () => {
        client.send("clear_patterns", {}, () => {
            console.log("Cleared Patterns");
        });
    });

    client.on("brightness", meta => {
        client.send("set_brightness", {brightness: meta}, res => {
            console.log(res);
        })
    })
}

main();