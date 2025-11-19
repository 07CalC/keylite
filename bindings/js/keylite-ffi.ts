

import ffi from "ffi-napi"
import ref from "ref-napi"

const voidPtr = ref.refType(ref.types.void);
const voidPtrPtr = ref.refType(voidPtr);
const ucharPtr = ref.refType(ref.types.uchar);
const ucharPtrPtr = ref.refType(ucharPtr);
const charPtr = ref.refType(ref.types.char);
const charPtrPtr = ref.refType(charPtr);
const sizeT = ref.types.size_t;
const sizeTPtr = ref.refType(sizeT);

const lib = ffi.Library(
  "../../ffi/libkeylite_kv.so",
  {
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
  })

export default lib;
