// Init rustlib
window.sks = require('sks-neon');

// Browser Code
window.lvl = function (name) {
    this.name = name;
    var self = this;

    this.board = document.createElement('canvas');
    this.board.contentEditable = true;
    this.board.id = this.name;
    this.board.width = 1920;
    this.board.height = 1080;
    this.board.style.cssText = 'width: 100%; border: 0px; outline: 0px;';
    this.board.style.cursor = 'pointer';

    this.levelBuilder = new window.sks.LevelBuilder(this.board);
    console.log(this.levelBuilder);

    let loopFunc = function () {
        if (self.levelBuilder.isDirty()) {
            let start = performance.now();
            self.levelBuilder.drawFrame();
            let end = performance.now();
            console.log("Dirty Redraw: ", end - start);
        }

        self.levelBuilder.update();
        requestAnimationFrame(loopFunc);
    }

    loopFunc();
}

lvl.prototype.exportLBL = function () {
    return this.levelBuilder.export('lbl');
}

lvl.prototype.exportPNG = function (cb) {
    return this.levelBuilder.getImage().toDataURL('image/png');
}

lvl.prototype.exportDev = function (num) {
    this.levelBuilder.setLevel(num);
    return this.levelBuilder.export('as3');
}

/*
function checkCtrlZ() {
if (window.lvl.zDown == true && window.lvl.ctrlDown == true && history.length > 0) {
var undo = level.history[level.history.length - 1];
console.log(undo);
level.render(undo.index, undo.oldBlock);
level.history.shift();
}
}
 */
// window.lvl.ctrlDown = false;
// window.lvl.zDown = false;
/*
document.onkeydown = function (event) {
switch (event.keyCode) {
case 17: {
window.lvl.ctrlDown = true;
break;
}
case 90: {
window.lvl.zDown = true;
break;
}
}
checkCtrlZ();
}
document.onkeyup = function (event) {
switch (event.keyCode) {
case 17: {
window.lvl.ctrlDown = false;
break;
}
case 90: {
window.lvl.zDown = false;
break;
}
}
checkCtrlZ();
}
 */

// Electron stuff
try {
    window.greenworks = require('greenworks');
} catch (e) {
    console.error('Greenworks dll error');
    greenworks = {
        initAPI: function () {
            return false;
        }
    };
}

window.path = require('path');
window.process = require('process');
window.remote = require('electron').remote;
window.fs = require('fs');

window.dialog = remote.dialog;
window.srcDir = process.cwd();

console.log('srcDir: ' + srcDir);
process.activateUvLoop();
window.onerror = function (errorMsg, url, lineNumber) {
    alert(errorMsg + ' line: ' + lineNumber);
};

// Steamworks init
window.steam = false;
if (greenworks.initAPI()) {
    console.log('Steamworks API Initialized');
    window.steam = true;

    let steamworksWorkshopSyncPath = path.resolve(srcDir, 'game/Custom Levels');

    if (!fs.existsSync(steamworksWorkshopSyncPath)) {
        try {
            fs.mkdirSync(steamworksWorkshopSyncPath);
        } catch (e) {
            alert(e);
            console.error(e);
        }
    }
    try {
        greenworks.ugcSynchronizeItems(steamworksWorkshopSyncPath, function (items) {
            console.log('Workshop Items Loaded: ');
            console.log(items);
        }, function (err) {
            throw err;
        });
    } catch (e) {
        alert(e);
    }
} else {
    console.log('Steamworks API Initialization Failed');
}

// Init
window.level = new lvl('build');

// Util
function readFile(path, encoding) {
    return new Promise((resolve, reject) => {
        window.fs.readFile(path, encoding, function (err, data) {
            if (err) {
                reject(err);
            } else {
                resolve(data);
            }
        });
    });
}

function getExportWindow() {
    let exportWindow = {};

    exportWindow.main = document.getElementById('export');

    exportWindow.dev = document.getElementById('export-dev');
    exportWindow.dev.num = document.getElementById('export-dev-num');
    exportWindow.dev.button = document.getElementById('export-dev-button');
    exportWindow.dev.titleBox = document.getElementById('export-dev-title');

    exportWindow.steam = document.getElementById('export-steam');
    exportWindow.steam.titleBox = document.getElementById('export-steam-title');
    exportWindow.steam.button = document.getElementById('export-steam-button');
    exportWindow.steam.description = document.getElementById('export-steam-description');
    exportWindow.steam.loading = document.getElementById('export-steam-loading');
    exportWindow.steam.loading.status = document.getElementById('export-steam-loading-status');
    exportWindow.steam.loading.spinner = document.getElementById('export-steam-loading-spinner');
    exportWindow.steam.loading.spinner.container = document.getElementById('export-steam-loading-spinner-container');
    exportWindow.steam.loading.button = document.getElementById('export-steam-loading-button');
    exportWindow.steam.loading.button.container = document.getElementById('export-steam-loading-button-container');

    exportWindow.lbl = document.getElementById('export-lbl');
    exportWindow.lbl.titleBox = document.getElementById('export-lbl-title');
    exportWindow.lbl.button = document.getElementById('export-lbl-button');

    if (!steam)
        exportWindow.steam.button.setAttribute('disabled', '');

    exportWindow.steam.loading.setLoaded = function () {
        exportWindow.steam.loading.status.innerHTML = 'Done';
        exportWindow.steam.loading.removeAttribute('modal');
        exportWindow.steam.loading.spinner.container.style.display = 'none';
        exportWindow.steam.loading.button.container.style.display = '';
        exportWindow.steam.loading.notifyResize();
    }

    exportWindow.steam.loading.reset = function () {
        exportWindow.steam.loading.status.innerHTML = 'Uploading Files to Steam Cloud...';
        exportWindow.steam.loading.setAttribute('modal', '');
        exportWindow.steam.loading.spinner.container.style.display = '';
        exportWindow.steam.loading.button.container.style.display = 'none';
        exportWindow.steam.loading.notifyResize();
    }

    exportWindow.closeAll = function () {
        exportWindow.main.close();
        exportWindow.steam.close();
        exportWindow.steam.loading.close();
        exportWindow.dev.close();
        exportWindow.lbl.close();
    }

    exportWindow.dev.saveFile = function () {
        let num = exportWindow.dev.num.value || 'x';
        let data = level.exportDev(num);
        remote.dialog.showSaveDialog({
            defaultPath: srcDir + '/' + exportWindow.dev.titleBox.value + '.txt'
        }, function (path) {
            if (path) {
                fs.writeFile(path, data, function (err) {
                    if (err) {
                        throw err; //TODO: Handle errors
                    } else {
                        console.log("File Saved!");
                        exportWindow.closeAll();
                    }
                });
            }
        });
    }

    exportWindow.lbl.saveFile = function () {
        let data = level.exportLBL();
        remote.dialog.showSaveDialog({
            defaultPath: srcDir + '/' + exportWindow.lbl.titleBox.value + '.txt'
        }, function (path) {
            if (path) {
                fs.writeFile(path, data, function (err) {
                    if (err) {
                        throw err; //TODO: Handle errors
                    } else {
                        console.log("File Saved!");
                        exportWindow.closeAll();
                    }
                });
            }
        });
    }

    exportWindow.steam.upload = function () {
        let title = exportWindow.steam.titleBox.value; //TODO: Validate param
        let description = exportWindow.steam.description.value;

        let levelPath = title + '.txt';
        let coverPath = title + '.png';
        let levelData = level.exportLBL();

        exportWindow.steam.loading.reset();
        exportWindow.closeAll();
        exportWindow.steam.loading.open();

        function onStatusUpdate(status) {
            let stat;
            switch (status) {
            case 'Completed on saving files on Steam Cloud.': {
                    stat = 'Sharing Files to Steam Cloud...';
                    break;
                }
            case 'Completed on sharing files.': {
                    stat = 'Uploading to Workshop...';
                    break;
                }
            }
            console.log('Workshop Upload Status: ' + status);
            exportWindow.steam.loading.status.innerHTML = stat;
            exportWindow.steam.loading.notifyResize();
        }

        function getLevelPNG() {
            return new Promise(function (resolve, reject) {
                resolve(level.exportPNG());
            });
        }

        function createFile(path, data, opts) {
            return new Promise(function (resolve, reject) {
                fs.writeFile(path, data, opts, function (err) {
                    if (err) {
                        return reject(err);
                    }
                    return resolve();
                });
            });
        }

        function publishSteamFile(levelPath, levelTitle, levelDescription, levelImagePath, update) {
            return new Promise(function (resolve, reject) {
                greenworks.ugcPublish(levelPath, levelTitle, levelDescription, levelImagePath, function (handle) {
                    resolve(handle);
                }, function (err) {
                    reject(err);
                }, function (msg) {
                    update(msg);
                });
            });
        }

        getLevelPNG().catch(function (err) {
            throw err;
        }).then(function (data) {
            console.log("Cover File Generated!");
            data = data.replace(/^data:image\/png;base64,/, "");
            return createFile(coverPath, data, 'base64');
        }).catch(function (err) {
            return err;
        }).then(function () {
            console.log("Cover File Created!");
            return createFile(levelPath, levelData);
        }).catch(function (err) {
            throw err;
        }).then(function () {
            console.log('Level File Saved!');
            return publishSteamFile(levelPath, title, description, coverPath, onStatusUpdate);
        }).catch(function (err) {
            throw err;
        }).then(function (handle) {
            console.log("Workshop File Uploaded!");
            console.log("Workshop File Handle: " + handle);
            exportWindow.steam.loading.setLoaded();
        });
    }

    return exportWindow;
}
