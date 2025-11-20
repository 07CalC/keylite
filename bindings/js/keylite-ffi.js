"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
var ffi_napi_1 = require("ffi-napi");
var ref_napi_1 = require("ref-napi");
var voidPtr = ref_napi_1.default.refType(ref_napi_1.default.types.void);
var voidPtrPtr = ref_napi_1.default.refType(voidPtr);
var ucharPtr = ref_napi_1.default.refType(ref_napi_1.default.types.uchar);
var ucharPtrPtr = ref_napi_1.default.refType(ucharPtr);
var charPtr = ref_napi_1.default.refType(ref_napi_1.default.types.char);
var charPtrPtr = ref_napi_1.default.refType(charPtr);
var sizeT = ref_napi_1.default.types.size_t;
var sizeTPtr = ref_napi_1.default.refType(sizeT);
var lib = ffi_napi_1.default.Library("../../ffi/libkeylite_kv.so", {
    keylite_open: ["int", ["string", voidPtrPtr]],
    keylite_close: ["void", [voidPtr]],
    keylite_put: ["int", [voidPtr, ucharPtr, sizeT, ucharPtr, sizeT]],
    keylite_get: ["int", [voidPtr, ucharPtr, sizeT, ucharPtrPtr, sizeTPtr]],
    keylite_del: ["int", [voidPtr, ucharPtr, sizeT]],
    keylite_free_value: ["void", [ucharPtr, sizeT]],
    keylite_put_str: ["int", [voidPtr, "string", "string"]],
    keylite_get_str: ["int", [voidPtr, "string", charPtrPtr]],
    keylite_del_str: ["int", [voidPtr, "string"]],
    keylite_free_str: ["void", [charPtr]],
    keylite_scan: ["int", [voidPtr, ucharPtr, sizeT, ucharPtr, sizeT, voidPtrPtr]],
    keylite_scan_str: ["int", [voidPtr, "string", "string", voidPtrPtr]],
    keylite_iter_next: ["int", [voidPtr, ucharPtrPtr, sizeTPtr, ucharPtrPtr, sizeTPtr]],
    keylite_iter_free: ["void", [voidPtr]]
});
exports.default = lib;
