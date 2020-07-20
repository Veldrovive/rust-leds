let WebSocket;
let shortid;
function isNode(){
	if (typeof window === 'undefined') {
		// Then we are most likely in node
		return true;
	} else {
		// Then we are likely in a browser
		return false;
	}
}
if (isNode()) {
    WebSocket = require('ws');
    shortid = require('shortid');
}

class Client {
    constructor(params) {
        ({
            url: this.url = "ws://localhost8000",
            userToken: this.uToken,
            deviceName: this.devName,
            appName: this.appName,
            errorCallback: this.errorCallback = (code, message) => {console.log("Encountered error", code, ":", message);},
            infoCallback: this.infoCallback = (message) => {console.log("Got info:", message);}
        } = params);
        this.ws = null;
        this.commandMap = {};
        this.responseMap = {};
    }

    static addUser(url, username, fullName, userToken) {
        const addUserReq = {
            "type": "addUser",
            "payload": {
                "uToken": userToken,
                "username": username,
                "fullName": fullName
            }
        }
        return new Promise((resolve, reject) => {
            const ws = new WebSocket(url);
            ws.on("open", () => {
                const timeout = setTimeout(() => {
                    reject("Add user request timed out");
                    ws.terminate();
                }, 2000)
                ws.on("message", data => {
                    data = JSON.parse(data);
                    const type = data.type;
                    const payload = data.payload;

                    if (type === "error") {
                        const { code, message } = payload;
                        if (code == 502) {
                            reject("User already exists");
                            clearTimeout(timeout);
                            ws.terminate();
                        }
                    }

                    if (type === "info") {
                        const { state, message } = payload;
                        if (state) {
                            // Then the new user was added
                            resolve(userToken);
                            clearTimeout(timeout);
                            ws.terminate();
                        }
                    }
                })
                ws.send(JSON.stringify(addUserReq));
            })
        })
    }

    static addDevice(url, userToken, deviceName) {
        const addDeviceReq = {
            "type": "addDevice",
            "payload": {
                "uToken": userToken,
                "devName": deviceName
            }
        }
        return new Promise((resolve, reject) => {
            const ws = new WebSocket(url);
            ws.on("open", () => {
                const timeout = setTimeout(() => {
                    reject("Add device request timed out");
                    ws.terminate();
                }, 2000)
                ws.on("message", data => {
                    data = JSON.parse(data);
                    const type = data.type;
                    const payload = data.payload;

                    if (type === "error") {
                        const { code, message } = payload;
                        if (code == 503) {
                            reject("Device already exists");
                            clearTimeout(timeout);
                            ws.terminate();
                        }
                    }

                    if (type === "info") {
                        const { state, message } = payload;
                        if (state) {
                            // Then the new device was added
                            resolve(deviceName);
                            clearTimeout(timeout);
                            ws.terminate();
                        }
                    }
                })
                ws.send(JSON.stringify(addDeviceReq));
            })
        })
    }

    _connect() {
        return new Promise((resolve, reject) => {
            const ws = new WebSocket(this.url);
            ws.on("open", () => {
                const registerReq = {
                    "type": "register",
                    "payload": {
                        "gToken": this.uToken,
                        "devName": this.devName,
                        "appName": this.appName
                    }
                }
                const timeout = setTimeout(() => {
                    reject("Register request timed out");
                }, 2000)
                ws.on("message", data => {
                    data = JSON.parse(data);
                    const type = data.type;
                    const payload = data.payload;
                    if (type === "registerResponse") {
                        clearTimeout(timeout);
                        if (payload.state) {
                            resolve(ws);
                        } else {
                            reject(payload.message);
                        }
                    }
                })
                ws.send(JSON.stringify(registerReq));
            })
        })
    }

    disconnect() {
        if (this.ws) {
            this.ws.terminate();
        }
    }

    static async setup(params){
        const client = new Client(params);
        try {
            client.ws = await client._connect();
        } catch(err) {
            console.log("Failed to register with the server");
            throw err;
        }
        client._setupMessageHandling();
		return client;
    }
    
    _setupMessageHandling() {
        this.ws.on("message", async data => {
            data = JSON.parse(data);
            const type = data.type;
            const payload = data.payload;

            if (type === "command") {
                this._handleCommand(payload.command, payload.meta, payload.callbackId);
            }

            if (type === "response") {
                this._handleResponse(payload.callbackId, payload.meta);
            }

            if (type === "error") {
                this.errorCallback(payload.code, payload.message);
            }

            if (type === "info") {
                this.infoCallback(payload.message);
            }
        })
    }

    async _handleCommand(command, meta, callbackId) {
        if (command in this.commandMap) {
            const res = await this.commandMap[command](meta)
            if (res !== undefined) {
                this._sendResponse(callbackId, res);
            }
        }
    }

    _sendResponse(callbackId, meta) {
        const responseReq = {
            "type": "response",
            "payload": {
                "callbackId": callbackId,
                "meta": meta
            }
        }
        this.ws.send(JSON.stringify(responseReq));
    }

    on(command, callback) {
        this.commandMap[command] = callback;
    }

    once(command, rawCallback) {
        const callback = async (meta) => {
            this.remove(command);
            return await rawCallback(meta);
        }
        this.on(command, callback);
    }

    remove(command) {
        delete this.commandMap[command];
    }

    send(command, meta, targets, mirror, responseCallback) {
        if (typeof targets === "function") {
            responseCallback = targets;
            targets = undefined;
            mirror = undefined;
        }
        if (typeof mirror === "function") {
            responseCallback = mirror;
            mirror = undefined;
        }
        if (typeof targets === "boolean") {
            mirror = targets;
            targets = undefined;
        }
        const callbackId = shortid.generate();
        const commandReq = {
            "type": "command",
            "payload": {
                "command": command,
                "meta": meta,
                "callbackId": callbackId,
                "targets": targets,
                "mirror": mirror
            }
        }
        this.ws.send(JSON.stringify(commandReq));
        if (typeof responseCallback === "function"){
            this.responseMap[callbackId] = responseCallback;
        }
    }

    _handleResponse(callbackId, meta) {
        if (callbackId in this.responseMap) {
            this.responseMap[callbackId](meta);
        }
    }
}

if (isNode()) {
	module.exports = Client;
}