"use strict";
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Keylite = exports.KeyliteIterator = void 0;
var keylite_ffi_1 = require("./keylite-ffi");
var ref_napi_1 = require("ref-napi");
var KeyliteResult;
(function (KeyliteResult) {
    KeyliteResult[KeyliteResult["Ok"] = 0] = "Ok";
    KeyliteResult[KeyliteResult["ErrNull"] = 1] = "ErrNull";
    KeyliteResult[KeyliteResult["ErrIo"] = 2] = "ErrIo";
    KeyliteResult[KeyliteResult["ErrUtf8"] = 3] = "ErrUtf8";
    KeyliteResult[KeyliteResult["ErrOther"] = 4] = "ErrOther";
})(KeyliteResult || (KeyliteResult = {}));
function checkResult(result, operation) {
    var _a;
    if (result !== KeyliteResult.Ok) {
        var errorMessages = (_a = {},
            _a[KeyliteResult.ErrNull] = "Null pointer error",
            _a[KeyliteResult.ErrIo] = "I/O error",
            _a[KeyliteResult.ErrUtf8] = "UTF-8 encoding error",
            _a[KeyliteResult.ErrOther] = "Unknown error",
            _a);
        throw new Error("".concat(operation, " failed: ").concat(errorMessages[result] || "Unknown error"));
    }
}
var KeyliteIterator = /** @class */ (function () {
    function KeyliteIterator(handle, db) {
        this.handle = handle;
        this.db = db;
    }
    KeyliteIterator.prototype.next = function () {
        var keyOut = ref_napi_1.default.alloc(ref_napi_1.default.refType(ref_napi_1.default.types.uchar));
        var keyLenOut = ref_napi_1.default.alloc(ref_napi_1.default.types.size_t);
        var valOut = ref_napi_1.default.alloc(ref_napi_1.default.refType(ref_napi_1.default.types.uchar));
        var valLenOut = ref_napi_1.default.alloc(ref_napi_1.default.types.size_t);
        var result = keylite_ffi_1.default.keylite_iter_next(this.handle, keyOut, keyLenOut, valOut, valLenOut);
        checkResult(result, "Iterator next");
        var keyPtr = keyOut.deref();
        var keyLen = keyLenOut.deref();
        // If keyPtr is null, we've reached the end
        if (ref_napi_1.default.isNull(keyPtr) || keyLen === 0) {
            return null;
        }
        var valPtr = valOut.deref();
        var valLen = valLenOut.deref();
        // Copy the data to JavaScript buffers
        var key = Buffer.from(ref_napi_1.default.reinterpret(keyPtr, keyLen, 0));
        var value = Buffer.from(ref_napi_1.default.reinterpret(valPtr, valLen, 0));
        // Free the C-allocated memory
        keylite_ffi_1.default.keylite_free_value(keyPtr, keyLen);
        keylite_ffi_1.default.keylite_free_value(valPtr, valLen);
        return { key: key, value: value };
    };
    KeyliteIterator.prototype.close = function () {
        if (this.handle) {
            keylite_ffi_1.default.keylite_iter_free(this.handle);
        }
    };
    // Iterator protocol for for...of loops
    KeyliteIterator.prototype[Symbol.iterator] = function () {
        var item;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    if (!((item = this.next()) !== null)) return [3 /*break*/, 2];
                    return [4 /*yield*/, item];
                case 1:
                    _a.sent();
                    return [3 /*break*/, 0];
                case 2:
                    this.close();
                    return [2 /*return*/];
            }
        });
    };
    return KeyliteIterator;
}());
exports.KeyliteIterator = KeyliteIterator;
var Keylite = /** @class */ (function () {
    function Keylite() {
        this.handle = null;
    }
    Keylite.prototype.open = function (path) {
        if (this.handle) {
            throw new Error("Database is already open");
        }
        var dbOut = ref_napi_1.default.alloc(ref_napi_1.default.refType(ref_napi_1.default.types.void));
        var result = keylite_ffi_1.default.keylite_open(path, dbOut);
        checkResult(result, "Open database");
        this.handle = dbOut.deref();
        if (ref_napi_1.default.isNull(this.handle)) {
            throw new Error("Failed to open database");
        }
    };
    Keylite.prototype.close = function () {
        if (!this.handle)
            return;
        keylite_ffi_1.default.keylite_close(this.handle);
        this.handle = null;
    };
    Keylite.prototype.ensureOpen = function () {
        if (!this.handle) {
            throw new Error("Database is not open");
        }
    };
    // Binary key-value operations
    Keylite.prototype.put = function (key, value) {
        this.ensureOpen();
        var result = keylite_ffi_1.default.keylite_put(this.handle, key, key.length, value, value.length);
        checkResult(result, "Put");
    };
    Keylite.prototype.get = function (key) {
        this.ensureOpen();
        var valOut = ref_napi_1.default.alloc(ref_napi_1.default.refType(ref_napi_1.default.types.uchar));
        var valLenOut = ref_napi_1.default.alloc(ref_napi_1.default.types.size_t);
        var result = keylite_ffi_1.default.keylite_get(this.handle, key, key.length, valOut, valLenOut);
        checkResult(result, "Get");
        var valPtr = valOut.deref();
        var valLen = valLenOut.deref();
        // If valPtr is null, key doesn't exist
        if (ref_napi_1.default.isNull(valPtr) || valLen === 0) {
            return null;
        }
        // Copy the data to a JavaScript buffer
        var value = Buffer.from(ref_napi_1.default.reinterpret(valPtr, valLen, 0));
        // Free the C-allocated memory
        keylite_ffi_1.default.keylite_free_value(valPtr, valLen);
        return value;
    };
    Keylite.prototype.del = function (key) {
        this.ensureOpen();
        var result = keylite_ffi_1.default.keylite_del(this.handle, key, key.length);
        checkResult(result, "Delete");
    };
    // String key-value operations
    Keylite.prototype.putStr = function (key, value) {
        this.ensureOpen();
        var result = keylite_ffi_1.default.keylite_put_str(this.handle, key, value);
        checkResult(result, "Put string");
    };
    Keylite.prototype.getStr = function (key) {
        this.ensureOpen();
        var valOut = ref_napi_1.default.alloc(ref_napi_1.default.refType(ref_napi_1.default.types.char));
        var result = keylite_ffi_1.default.keylite_get_str(this.handle, key, valOut);
        checkResult(result, "Get string");
        var valPtr = valOut.deref();
        // If valPtr is null, key doesn't exist
        if (ref_napi_1.default.isNull(valPtr)) {
            return null;
        }
        // Read the string - readCString reads until null terminator
        var value = ref_napi_1.default.readCString(valPtr, 0);
        // Free the C-allocated memory
        keylite_ffi_1.default.keylite_free_str(valPtr);
        return value;
    };
    Keylite.prototype.delStr = function (key) {
        this.ensureOpen();
        var result = keylite_ffi_1.default.keylite_del_str(this.handle, key);
        checkResult(result, "Delete string");
    };
    // Scan operations
    Keylite.prototype.scan = function (start, end) {
        this.ensureOpen();
        var iterOut = ref_napi_1.default.alloc(ref_napi_1.default.refType(ref_napi_1.default.types.void));
        var startPtr = start ? start : null;
        var startLen = start ? start.length : 0;
        var endPtr = end ? end : null;
        var endLen = end ? end.length : 0;
        var result = keylite_ffi_1.default.keylite_scan(this.handle, startPtr, startLen, endPtr, endLen, iterOut);
        checkResult(result, "Scan");
        var iterHandle = iterOut.deref();
        return new KeyliteIterator(iterHandle, this);
    };
    Keylite.prototype.scanStr = function (start, end) {
        this.ensureOpen();
        var iterOut = ref_napi_1.default.alloc(ref_napi_1.default.refType(ref_napi_1.default.types.void));
        var result = keylite_ffi_1.default.keylite_scan_str(this.handle, start || null, end || null, iterOut);
        checkResult(result, "Scan string");
        var iterHandle = iterOut.deref();
        return new KeyliteIterator(iterHandle, this);
    };
    return Keylite;
}());
exports.Keylite = Keylite;
