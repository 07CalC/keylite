import { createRequire } from "node:module";
var __create = Object.create;
var __getProtoOf = Object.getPrototypeOf;
var __defProp = Object.defineProperty;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __toESM = (mod, isNodeMode, target) => {
  target = mod != null ? __create(__getProtoOf(mod)) : {};
  const to = isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target;
  for (let key of __getOwnPropNames(mod))
    if (!__hasOwnProp.call(to, key))
      __defProp(to, key, {
        get: () => mod[key],
        enumerable: true
      });
  return to;
};
var __commonJS = (cb, mod) => () => (mod || cb((mod = { exports: {} }).exports, mod), mod.exports);
var __require = /* @__PURE__ */ createRequire(import.meta.url);

// node_modules/ms/index.js
var require_ms = __commonJS((exports, module) => {
  var s = 1000;
  var m = s * 60;
  var h = m * 60;
  var d = h * 24;
  var w = d * 7;
  var y = d * 365.25;
  module.exports = function(val, options) {
    options = options || {};
    var type = typeof val;
    if (type === "string" && val.length > 0) {
      return parse(val);
    } else if (type === "number" && isFinite(val)) {
      return options.long ? fmtLong(val) : fmtShort(val);
    }
    throw new Error("val is not a non-empty string or a valid number. val=" + JSON.stringify(val));
  };
  function parse(str) {
    str = String(str);
    if (str.length > 100) {
      return;
    }
    var match = /^(-?(?:\d+)?\.?\d+) *(milliseconds?|msecs?|ms|seconds?|secs?|s|minutes?|mins?|m|hours?|hrs?|h|days?|d|weeks?|w|years?|yrs?|y)?$/i.exec(str);
    if (!match) {
      return;
    }
    var n = parseFloat(match[1]);
    var type = (match[2] || "ms").toLowerCase();
    switch (type) {
      case "years":
      case "year":
      case "yrs":
      case "yr":
      case "y":
        return n * y;
      case "weeks":
      case "week":
      case "w":
        return n * w;
      case "days":
      case "day":
      case "d":
        return n * d;
      case "hours":
      case "hour":
      case "hrs":
      case "hr":
      case "h":
        return n * h;
      case "minutes":
      case "minute":
      case "mins":
      case "min":
      case "m":
        return n * m;
      case "seconds":
      case "second":
      case "secs":
      case "sec":
      case "s":
        return n * s;
      case "milliseconds":
      case "millisecond":
      case "msecs":
      case "msec":
      case "ms":
        return n;
      default:
        return;
    }
  }
  function fmtShort(ms) {
    var msAbs = Math.abs(ms);
    if (msAbs >= d) {
      return Math.round(ms / d) + "d";
    }
    if (msAbs >= h) {
      return Math.round(ms / h) + "h";
    }
    if (msAbs >= m) {
      return Math.round(ms / m) + "m";
    }
    if (msAbs >= s) {
      return Math.round(ms / s) + "s";
    }
    return ms + "ms";
  }
  function fmtLong(ms) {
    var msAbs = Math.abs(ms);
    if (msAbs >= d) {
      return plural(ms, msAbs, d, "day");
    }
    if (msAbs >= h) {
      return plural(ms, msAbs, h, "hour");
    }
    if (msAbs >= m) {
      return plural(ms, msAbs, m, "minute");
    }
    if (msAbs >= s) {
      return plural(ms, msAbs, s, "second");
    }
    return ms + " ms";
  }
  function plural(ms, msAbs, n, name) {
    var isPlural = msAbs >= n * 1.5;
    return Math.round(ms / n) + " " + name + (isPlural ? "s" : "");
  }
});

// node_modules/debug/src/common.js
var require_common = __commonJS((exports, module) => {
  function setup(env) {
    createDebug.debug = createDebug;
    createDebug.default = createDebug;
    createDebug.coerce = coerce;
    createDebug.disable = disable;
    createDebug.enable = enable;
    createDebug.enabled = enabled;
    createDebug.humanize = require_ms();
    createDebug.destroy = destroy;
    Object.keys(env).forEach((key) => {
      createDebug[key] = env[key];
    });
    createDebug.names = [];
    createDebug.skips = [];
    createDebug.formatters = {};
    function selectColor(namespace) {
      let hash = 0;
      for (let i = 0;i < namespace.length; i++) {
        hash = (hash << 5) - hash + namespace.charCodeAt(i);
        hash |= 0;
      }
      return createDebug.colors[Math.abs(hash) % createDebug.colors.length];
    }
    createDebug.selectColor = selectColor;
    function createDebug(namespace) {
      let prevTime;
      let enableOverride = null;
      let namespacesCache;
      let enabledCache;
      function debug(...args) {
        if (!debug.enabled) {
          return;
        }
        const self = debug;
        const curr = Number(new Date);
        const ms = curr - (prevTime || curr);
        self.diff = ms;
        self.prev = prevTime;
        self.curr = curr;
        prevTime = curr;
        args[0] = createDebug.coerce(args[0]);
        if (typeof args[0] !== "string") {
          args.unshift("%O");
        }
        let index = 0;
        args[0] = args[0].replace(/%([a-zA-Z%])/g, (match, format) => {
          if (match === "%%") {
            return "%";
          }
          index++;
          const formatter = createDebug.formatters[format];
          if (typeof formatter === "function") {
            const val = args[index];
            match = formatter.call(self, val);
            args.splice(index, 1);
            index--;
          }
          return match;
        });
        createDebug.formatArgs.call(self, args);
        const logFn = self.log || createDebug.log;
        logFn.apply(self, args);
      }
      debug.namespace = namespace;
      debug.useColors = createDebug.useColors();
      debug.color = createDebug.selectColor(namespace);
      debug.extend = extend;
      debug.destroy = createDebug.destroy;
      Object.defineProperty(debug, "enabled", {
        enumerable: true,
        configurable: false,
        get: () => {
          if (enableOverride !== null) {
            return enableOverride;
          }
          if (namespacesCache !== createDebug.namespaces) {
            namespacesCache = createDebug.namespaces;
            enabledCache = createDebug.enabled(namespace);
          }
          return enabledCache;
        },
        set: (v) => {
          enableOverride = v;
        }
      });
      if (typeof createDebug.init === "function") {
        createDebug.init(debug);
      }
      return debug;
    }
    function extend(namespace, delimiter) {
      const newDebug = createDebug(this.namespace + (typeof delimiter === "undefined" ? ":" : delimiter) + namespace);
      newDebug.log = this.log;
      return newDebug;
    }
    function enable(namespaces) {
      createDebug.save(namespaces);
      createDebug.namespaces = namespaces;
      createDebug.names = [];
      createDebug.skips = [];
      const split = (typeof namespaces === "string" ? namespaces : "").trim().replace(/\s+/g, ",").split(",").filter(Boolean);
      for (const ns of split) {
        if (ns[0] === "-") {
          createDebug.skips.push(ns.slice(1));
        } else {
          createDebug.names.push(ns);
        }
      }
    }
    function matchesTemplate(search, template) {
      let searchIndex = 0;
      let templateIndex = 0;
      let starIndex = -1;
      let matchIndex = 0;
      while (searchIndex < search.length) {
        if (templateIndex < template.length && (template[templateIndex] === search[searchIndex] || template[templateIndex] === "*")) {
          if (template[templateIndex] === "*") {
            starIndex = templateIndex;
            matchIndex = searchIndex;
            templateIndex++;
          } else {
            searchIndex++;
            templateIndex++;
          }
        } else if (starIndex !== -1) {
          templateIndex = starIndex + 1;
          matchIndex++;
          searchIndex = matchIndex;
        } else {
          return false;
        }
      }
      while (templateIndex < template.length && template[templateIndex] === "*") {
        templateIndex++;
      }
      return templateIndex === template.length;
    }
    function disable() {
      const namespaces = [
        ...createDebug.names,
        ...createDebug.skips.map((namespace) => "-" + namespace)
      ].join(",");
      createDebug.enable("");
      return namespaces;
    }
    function enabled(name) {
      for (const skip of createDebug.skips) {
        if (matchesTemplate(name, skip)) {
          return false;
        }
      }
      for (const ns of createDebug.names) {
        if (matchesTemplate(name, ns)) {
          return true;
        }
      }
      return false;
    }
    function coerce(val) {
      if (val instanceof Error) {
        return val.stack || val.message;
      }
      return val;
    }
    function destroy() {
      console.warn("Instance method `debug.destroy()` is deprecated and no longer does anything. It will be removed in the next major version of `debug`.");
    }
    createDebug.enable(createDebug.load());
    return createDebug;
  }
  module.exports = setup;
});

// node_modules/debug/src/browser.js
var require_browser = __commonJS((exports, module) => {
  exports.formatArgs = formatArgs;
  exports.save = save;
  exports.load = load;
  exports.useColors = useColors;
  exports.storage = localstorage();
  exports.destroy = (() => {
    let warned = false;
    return () => {
      if (!warned) {
        warned = true;
        console.warn("Instance method `debug.destroy()` is deprecated and no longer does anything. It will be removed in the next major version of `debug`.");
      }
    };
  })();
  exports.colors = [
    "#0000CC",
    "#0000FF",
    "#0033CC",
    "#0033FF",
    "#0066CC",
    "#0066FF",
    "#0099CC",
    "#0099FF",
    "#00CC00",
    "#00CC33",
    "#00CC66",
    "#00CC99",
    "#00CCCC",
    "#00CCFF",
    "#3300CC",
    "#3300FF",
    "#3333CC",
    "#3333FF",
    "#3366CC",
    "#3366FF",
    "#3399CC",
    "#3399FF",
    "#33CC00",
    "#33CC33",
    "#33CC66",
    "#33CC99",
    "#33CCCC",
    "#33CCFF",
    "#6600CC",
    "#6600FF",
    "#6633CC",
    "#6633FF",
    "#66CC00",
    "#66CC33",
    "#9900CC",
    "#9900FF",
    "#9933CC",
    "#9933FF",
    "#99CC00",
    "#99CC33",
    "#CC0000",
    "#CC0033",
    "#CC0066",
    "#CC0099",
    "#CC00CC",
    "#CC00FF",
    "#CC3300",
    "#CC3333",
    "#CC3366",
    "#CC3399",
    "#CC33CC",
    "#CC33FF",
    "#CC6600",
    "#CC6633",
    "#CC9900",
    "#CC9933",
    "#CCCC00",
    "#CCCC33",
    "#FF0000",
    "#FF0033",
    "#FF0066",
    "#FF0099",
    "#FF00CC",
    "#FF00FF",
    "#FF3300",
    "#FF3333",
    "#FF3366",
    "#FF3399",
    "#FF33CC",
    "#FF33FF",
    "#FF6600",
    "#FF6633",
    "#FF9900",
    "#FF9933",
    "#FFCC00",
    "#FFCC33"
  ];
  function useColors() {
    if (typeof window !== "undefined" && window.process && (window.process.type === "renderer" || window.process.__nwjs)) {
      return true;
    }
    if (typeof navigator !== "undefined" && navigator.userAgent && navigator.userAgent.toLowerCase().match(/(edge|trident)\/(\d+)/)) {
      return false;
    }
    let m;
    return typeof document !== "undefined" && document.documentElement && document.documentElement.style && document.documentElement.style.WebkitAppearance || typeof window !== "undefined" && window.console && (window.console.firebug || window.console.exception && window.console.table) || typeof navigator !== "undefined" && navigator.userAgent && (m = navigator.userAgent.toLowerCase().match(/firefox\/(\d+)/)) && parseInt(m[1], 10) >= 31 || typeof navigator !== "undefined" && navigator.userAgent && navigator.userAgent.toLowerCase().match(/applewebkit\/(\d+)/);
  }
  function formatArgs(args) {
    args[0] = (this.useColors ? "%c" : "") + this.namespace + (this.useColors ? " %c" : " ") + args[0] + (this.useColors ? "%c " : " ") + "+" + module.exports.humanize(this.diff);
    if (!this.useColors) {
      return;
    }
    const c = "color: " + this.color;
    args.splice(1, 0, c, "color: inherit");
    let index = 0;
    let lastC = 0;
    args[0].replace(/%[a-zA-Z%]/g, (match) => {
      if (match === "%%") {
        return;
      }
      index++;
      if (match === "%c") {
        lastC = index;
      }
    });
    args.splice(lastC, 0, c);
  }
  exports.log = console.debug || console.log || (() => {});
  function save(namespaces) {
    try {
      if (namespaces) {
        exports.storage.setItem("debug", namespaces);
      } else {
        exports.storage.removeItem("debug");
      }
    } catch (error) {}
  }
  function load() {
    let r;
    try {
      r = exports.storage.getItem("debug") || exports.storage.getItem("DEBUG");
    } catch (error) {}
    if (!r && typeof process !== "undefined" && "env" in process) {
      r = process.env.DEBUG;
    }
    return r;
  }
  function localstorage() {
    try {
      return localStorage;
    } catch (error) {}
  }
  module.exports = require_common()(exports);
  var { formatters } = module.exports;
  formatters.j = function(v) {
    try {
      return JSON.stringify(v);
    } catch (error) {
      return "[UnexpectedJSONParseError]: " + error.message;
    }
  };
});

// node_modules/debug/src/node.js
var require_node = __commonJS((exports, module) => {
  var tty = __require("tty");
  var util = __require("util");
  exports.init = init;
  exports.log = log;
  exports.formatArgs = formatArgs;
  exports.save = save;
  exports.load = load;
  exports.useColors = useColors;
  exports.destroy = util.deprecate(() => {}, "Instance method `debug.destroy()` is deprecated and no longer does anything. It will be removed in the next major version of `debug`.");
  exports.colors = [6, 2, 3, 4, 5, 1];
  try {
    const supportsColor = (()=>{throw new Error("Cannot require module "+"supports-color");})();
    if (supportsColor && (supportsColor.stderr || supportsColor).level >= 2) {
      exports.colors = [
        20,
        21,
        26,
        27,
        32,
        33,
        38,
        39,
        40,
        41,
        42,
        43,
        44,
        45,
        56,
        57,
        62,
        63,
        68,
        69,
        74,
        75,
        76,
        77,
        78,
        79,
        80,
        81,
        92,
        93,
        98,
        99,
        112,
        113,
        128,
        129,
        134,
        135,
        148,
        149,
        160,
        161,
        162,
        163,
        164,
        165,
        166,
        167,
        168,
        169,
        170,
        171,
        172,
        173,
        178,
        179,
        184,
        185,
        196,
        197,
        198,
        199,
        200,
        201,
        202,
        203,
        204,
        205,
        206,
        207,
        208,
        209,
        214,
        215,
        220,
        221
      ];
    }
  } catch (error) {}
  exports.inspectOpts = Object.keys(process.env).filter((key) => {
    return /^debug_/i.test(key);
  }).reduce((obj, key) => {
    const prop = key.substring(6).toLowerCase().replace(/_([a-z])/g, (_, k) => {
      return k.toUpperCase();
    });
    let val = process.env[key];
    if (/^(yes|on|true|enabled)$/i.test(val)) {
      val = true;
    } else if (/^(no|off|false|disabled)$/i.test(val)) {
      val = false;
    } else if (val === "null") {
      val = null;
    } else {
      val = Number(val);
    }
    obj[prop] = val;
    return obj;
  }, {});
  function useColors() {
    return "colors" in exports.inspectOpts ? Boolean(exports.inspectOpts.colors) : tty.isatty(process.stderr.fd);
  }
  function formatArgs(args) {
    const { namespace: name, useColors: useColors2 } = this;
    if (useColors2) {
      const c = this.color;
      const colorCode = "\x1B[3" + (c < 8 ? c : "8;5;" + c);
      const prefix = `  ${colorCode};1m${name} \x1B[0m`;
      args[0] = prefix + args[0].split(`
`).join(`
` + prefix);
      args.push(colorCode + "m+" + module.exports.humanize(this.diff) + "\x1B[0m");
    } else {
      args[0] = getDate() + name + " " + args[0];
    }
  }
  function getDate() {
    if (exports.inspectOpts.hideDate) {
      return "";
    }
    return new Date().toISOString() + " ";
  }
  function log(...args) {
    return process.stderr.write(util.formatWithOptions(exports.inspectOpts, ...args) + `
`);
  }
  function save(namespaces) {
    if (namespaces) {
      process.env.DEBUG = namespaces;
    } else {
      delete process.env.DEBUG;
    }
  }
  function load() {
    return process.env.DEBUG;
  }
  function init(debug) {
    debug.inspectOpts = {};
    const keys = Object.keys(exports.inspectOpts);
    for (let i = 0;i < keys.length; i++) {
      debug.inspectOpts[keys[i]] = exports.inspectOpts[keys[i]];
    }
  }
  module.exports = require_common()(exports);
  var { formatters } = module.exports;
  formatters.o = function(v) {
    this.inspectOpts.colors = this.useColors;
    return util.inspect(v, this.inspectOpts).split(`
`).map((str) => str.trim()).join(" ");
  };
  formatters.O = function(v) {
    this.inspectOpts.colors = this.useColors;
    return util.inspect(v, this.inspectOpts);
  };
});

// node_modules/debug/src/index.js
var require_src = __commonJS((exports, module) => {
  if (typeof process === "undefined" || process.type === "renderer" || false || process.__nwjs) {
    module.exports = require_browser();
  } else {
    module.exports = require_node();
  }
});

// node_modules/node-gyp-build/node-gyp-build.js
var require_node_gyp_build = __commonJS((exports, module) => {
  var fs = __require("fs");
  var path = __require("path");
  var os = __require("os");
  var runtimeRequire = typeof __webpack_require__ === "function" ? __non_webpack_require__ : __require;
  var vars = process.config && process.config.variables || {};
  var prebuildsOnly = !!process.env.PREBUILDS_ONLY;
  var abi = process.versions.modules;
  var runtime = isElectron() ? "electron" : isNwjs() ? "node-webkit" : "node";
  var arch = process.env.npm_config_arch || os.arch();
  var platform = process.env.npm_config_platform || os.platform();
  var libc = process.env.LIBC || (isAlpine(platform) ? "musl" : "glibc");
  var armv = process.env.ARM_VERSION || (arch === "arm64" ? "8" : vars.arm_version) || "";
  var uv = (process.versions.uv || "").split(".")[0];
  module.exports = load;
  function load(dir) {
    return runtimeRequire(load.resolve(dir));
  }
  load.resolve = load.path = function(dir) {
    dir = path.resolve(dir || ".");
    try {
      var name = runtimeRequire(path.join(dir, "package.json")).name.toUpperCase().replace(/-/g, "_");
      if (process.env[name + "_PREBUILD"])
        dir = process.env[name + "_PREBUILD"];
    } catch (err) {}
    if (!prebuildsOnly) {
      var release = getFirst(path.join(dir, "build/Release"), matchBuild);
      if (release)
        return release;
      var debug = getFirst(path.join(dir, "build/Debug"), matchBuild);
      if (debug)
        return debug;
    }
    var prebuild = resolve(dir);
    if (prebuild)
      return prebuild;
    var nearby = resolve(path.dirname(process.execPath));
    if (nearby)
      return nearby;
    var target = [
      "platform=" + platform,
      "arch=" + arch,
      "runtime=" + runtime,
      "abi=" + abi,
      "uv=" + uv,
      armv ? "armv=" + armv : "",
      "libc=" + libc,
      "node=" + process.versions.node,
      process.versions.electron ? "electron=" + process.versions.electron : "",
      typeof __webpack_require__ === "function" ? "webpack=true" : ""
    ].filter(Boolean).join(" ");
    throw new Error("No native build was found for " + target + `
    loaded from: ` + dir + `
`);
    function resolve(dir2) {
      var tuples = readdirSync(path.join(dir2, "prebuilds")).map(parseTuple);
      var tuple = tuples.filter(matchTuple(platform, arch)).sort(compareTuples)[0];
      if (!tuple)
        return;
      var prebuilds = path.join(dir2, "prebuilds", tuple.name);
      var parsed = readdirSync(prebuilds).map(parseTags);
      var candidates = parsed.filter(matchTags(runtime, abi));
      var winner = candidates.sort(compareTags(runtime))[0];
      if (winner)
        return path.join(prebuilds, winner.file);
    }
  };
  function readdirSync(dir) {
    try {
      return fs.readdirSync(dir);
    } catch (err) {
      return [];
    }
  }
  function getFirst(dir, filter) {
    var files = readdirSync(dir).filter(filter);
    return files[0] && path.join(dir, files[0]);
  }
  function matchBuild(name) {
    return /\.node$/.test(name);
  }
  function parseTuple(name) {
    var arr = name.split("-");
    if (arr.length !== 2)
      return;
    var platform2 = arr[0];
    var architectures = arr[1].split("+");
    if (!platform2)
      return;
    if (!architectures.length)
      return;
    if (!architectures.every(Boolean))
      return;
    return { name, platform: platform2, architectures };
  }
  function matchTuple(platform2, arch2) {
    return function(tuple) {
      if (tuple == null)
        return false;
      if (tuple.platform !== platform2)
        return false;
      return tuple.architectures.includes(arch2);
    };
  }
  function compareTuples(a, b) {
    return a.architectures.length - b.architectures.length;
  }
  function parseTags(file) {
    var arr = file.split(".");
    var extension = arr.pop();
    var tags = { file, specificity: 0 };
    if (extension !== "node")
      return;
    for (var i = 0;i < arr.length; i++) {
      var tag = arr[i];
      if (tag === "node" || tag === "electron" || tag === "node-webkit") {
        tags.runtime = tag;
      } else if (tag === "napi") {
        tags.napi = true;
      } else if (tag.slice(0, 3) === "abi") {
        tags.abi = tag.slice(3);
      } else if (tag.slice(0, 2) === "uv") {
        tags.uv = tag.slice(2);
      } else if (tag.slice(0, 4) === "armv") {
        tags.armv = tag.slice(4);
      } else if (tag === "glibc" || tag === "musl") {
        tags.libc = tag;
      } else {
        continue;
      }
      tags.specificity++;
    }
    return tags;
  }
  function matchTags(runtime2, abi2) {
    return function(tags) {
      if (tags == null)
        return false;
      if (tags.runtime && tags.runtime !== runtime2 && !runtimeAgnostic(tags))
        return false;
      if (tags.abi && tags.abi !== abi2 && !tags.napi)
        return false;
      if (tags.uv && tags.uv !== uv)
        return false;
      if (tags.armv && tags.armv !== armv)
        return false;
      if (tags.libc && tags.libc !== libc)
        return false;
      return true;
    };
  }
  function runtimeAgnostic(tags) {
    return tags.runtime === "node" && tags.napi;
  }
  function compareTags(runtime2) {
    return function(a, b) {
      if (a.runtime !== b.runtime) {
        return a.runtime === runtime2 ? -1 : 1;
      } else if (a.abi !== b.abi) {
        return a.abi ? -1 : 1;
      } else if (a.specificity !== b.specificity) {
        return a.specificity > b.specificity ? -1 : 1;
      } else {
        return 0;
      }
    };
  }
  function isNwjs() {
    return !!(process.versions && process.versions.nw);
  }
  function isElectron() {
    if (process.versions && process.versions.electron)
      return true;
    if (process.env.ELECTRON_RUN_AS_NODE)
      return true;
    return typeof window !== "undefined" && window.process && window.process.type === "renderer";
  }
  function isAlpine(platform2) {
    return platform2 === "linux" && fs.existsSync("/etc/alpine-release");
  }
  load.parseTags = parseTags;
  load.matchTags = matchTags;
  load.compareTags = compareTags;
  load.parseTuple = parseTuple;
  load.matchTuple = matchTuple;
  load.compareTuples = compareTuples;
});

// node_modules/node-gyp-build/index.js
var require_node_gyp_build2 = __commonJS((exports, module) => {
  var runtimeRequire = typeof __webpack_require__ === "function" ? __non_webpack_require__ : __require;
  if (typeof runtimeRequire.addon === "function") {
    module.exports = runtimeRequire.addon.bind(runtimeRequire);
  } else {
    module.exports = require_node_gyp_build();
  }
});

// node_modules/ref-napi/lib/ref.js
var require_ref = __commonJS((exports, module) => {
  var __dirname = "/home/calc/Documents/keylite/bindings/js/node_modules/ref-napi/lib";
  var assert = __require("assert");
  var inspect = __require("util").inspect;
  var debug = require_src()("ref");
  var os = __require("os");
  var path = __require("path");
  exports = module.exports = require_node_gyp_build2()(path.join(__dirname, ".."));
  exports.endianness = os.endianness();
  exports.refType = function refType(type) {
    const _type = exports.coerceType(type);
    const rtn = Object.create(_type);
    rtn.indirection++;
    if (_type.name) {
      Object.defineProperty(rtn, "name", {
        value: _type.name + "*",
        configurable: true,
        enumerable: true,
        writable: true
      });
    }
    return rtn;
  };
  exports.derefType = function derefType(type) {
    const _type = exports.coerceType(type);
    if (_type.indirection === 1) {
      throw new Error("Cannot create deref'd type for type with indirection 1");
    }
    let rtn = Object.getPrototypeOf(_type);
    if (rtn.indirection !== _type.indirection - 1) {
      rtn = Object.create(_type);
      rtn.indirection--;
    }
    return rtn;
  };
  exports.coerceType = function coerceType(type) {
    let rtn = type;
    if (typeof rtn === "string") {
      rtn = exports.types[type];
      if (rtn)
        return rtn;
      rtn = type.replace(/\s+/g, "").toLowerCase();
      if (rtn === "pointer") {
        rtn = exports.refType(exports.types.void);
      } else if (rtn === "string") {
        rtn = exports.types.CString;
      } else {
        var refCount = 0;
        rtn = rtn.replace(/\*/g, function() {
          refCount++;
          return "";
        });
        rtn = exports.types[rtn];
        if (refCount > 0) {
          if (!(rtn && ("size" in rtn) && ("indirection" in rtn))) {
            throw new TypeError('could not determine a proper "type" from: ' + inspect(type));
          }
          for (let i = 0;i < refCount; i++) {
            rtn = exports.refType(rtn);
          }
        }
      }
    }
    if (!(rtn && ("size" in rtn) && ("indirection" in rtn))) {
      throw new TypeError('could not determine a proper "type" from: ' + inspect(type));
    }
    return rtn;
  };
  exports.getType = function getType(buffer) {
    if (!buffer.type) {
      debug('WARN: no "type" found on buffer, setting default "type"', buffer);
      buffer.type = {};
      buffer.type.size = buffer.length;
      buffer.type.indirection = 1;
      buffer.type.get = function get() {
        throw new Error('unknown "type"; cannot get()');
      };
      buffer.type.set = function set() {
        throw new Error('unknown "type"; cannot set()');
      };
    }
    return exports.coerceType(buffer.type);
  };
  exports.get = function get(buffer, offset, type) {
    if (!offset) {
      offset = 0;
    }
    if (type) {
      type = exports.coerceType(type);
    } else {
      type = exports.getType(buffer);
    }
    debug("get(): (offset: %d)", offset, buffer);
    assert(type.indirection > 0, `"indirection" level must be at least 1, saw ${type.indirection}`);
    if (type.indirection === 1) {
      return type.get(buffer, offset);
    } else {
      const size = type.indirection === 2 ? type.size : exports.sizeof.pointer;
      const reference = exports.readPointer(buffer, offset, size);
      reference.type = exports.derefType(type);
      return reference;
    }
  };
  exports.set = function set(buffer, offset, value, type) {
    if (!offset) {
      offset = 0;
    }
    if (type) {
      type = exports.coerceType(type);
    } else {
      type = exports.getType(buffer);
    }
    debug("set(): (offset: %d)", offset, buffer, value);
    assert(type.indirection >= 1, '"indirection" level must be at least 1');
    if (type.indirection === 1) {
      type.set(buffer, offset, value);
    } else {
      exports.writePointer(buffer, offset, value);
    }
  };
  exports.alloc = function alloc(_type, value) {
    var type = exports.coerceType(_type);
    debug('allocating Buffer for type with "size"', type.size);
    let size;
    if (type.indirection === 1) {
      size = type.size;
    } else {
      size = exports.sizeof.pointer;
    }
    const buffer = Buffer.alloc(size);
    buffer.type = type;
    if (arguments.length >= 2) {
      debug("setting value on allocated buffer", value);
      exports.set(buffer, 0, value, type);
    }
    return buffer;
  };
  exports.allocCString = function allocCString(string, encoding) {
    if (string == null || Buffer.isBuffer(string) && exports.isNull(string)) {
      return exports.NULL;
    }
    const size = Buffer.byteLength(string, encoding) + 1;
    const buffer = Buffer.allocUnsafe(size);
    exports.writeCString(buffer, 0, string, encoding);
    buffer.type = charPtrType;
    return buffer;
  };
  exports.writeCString = function writeCString(buffer, offset, string, encoding) {
    assert(Buffer.isBuffer(buffer), "expected a Buffer as the first argument");
    assert.strictEqual("string", typeof string, 'expected a "string" as the third argument');
    if (!offset) {
      offset = 0;
    }
    if (!encoding) {
      encoding = "utf8";
    }
    const size = buffer.length - offset - 1;
    const len = buffer.write(string, offset, size, encoding);
    buffer.writeUInt8(0, offset + len);
  };
  exports["readInt64" + exports.endianness] = exports.readInt64;
  exports["readUInt64" + exports.endianness] = exports.readUInt64;
  exports["writeInt64" + exports.endianness] = exports.writeInt64;
  exports["writeUInt64" + exports.endianness] = exports.writeUInt64;
  var opposite = exports.endianness == "LE" ? "BE" : "LE";
  var int64temp = Buffer.alloc(exports.sizeof.int64);
  var uint64temp = Buffer.alloc(exports.sizeof.uint64);
  exports["readInt64" + opposite] = function(buffer, offset) {
    for (let i = 0;i < exports.sizeof.int64; i++) {
      int64temp[i] = buffer[offset + exports.sizeof.int64 - i - 1];
    }
    return exports.readInt64(int64temp, 0);
  };
  exports["readUInt64" + opposite] = function(buffer, offset) {
    for (let i = 0;i < exports.sizeof.uint64; i++) {
      uint64temp[i] = buffer[offset + exports.sizeof.uint64 - i - 1];
    }
    return exports.readUInt64(uint64temp, 0);
  };
  exports["writeInt64" + opposite] = function(buffer, offset, value) {
    exports.writeInt64(int64temp, 0, value);
    for (let i = 0;i < exports.sizeof.int64; i++) {
      buffer[offset + i] = int64temp[exports.sizeof.int64 - i - 1];
    }
  };
  exports["writeUInt64" + opposite] = function(buffer, offset, value) {
    exports.writeUInt64(uint64temp, 0, value);
    for (let i = 0;i < exports.sizeof.uint64; i++) {
      buffer[offset + i] = uint64temp[exports.sizeof.uint64 - i - 1];
    }
  };
  exports.ref = function ref(buffer) {
    debug("creating a reference to buffer", buffer);
    var type = exports.refType(exports.getType(buffer));
    return exports.alloc(type, buffer);
  };
  exports.deref = function deref(buffer) {
    debug("dereferencing buffer", buffer);
    return exports.get(buffer);
  };
  var kAttachedRefs = Symbol("attached");
  exports._attach = function _attach(buf, obj) {
    if (!buf[kAttachedRefs]) {
      buf[kAttachedRefs] = [];
    }
    buf[kAttachedRefs].push(obj);
  };
  exports.writeObject = function writeObject(buf, offset, obj) {
    debug("writing Object to buffer", buf, offset, obj);
    exports._writeObject(buf, offset, obj);
    exports._attach(buf, obj);
  };
  exports.writePointer = function writePointer(buf, offset, ptr) {
    debug("writing pointer to buffer", buf, offset, ptr);
    exports._writePointer(buf, offset, ptr, true);
  };
  exports.reinterpret = function reinterpret(buffer, size, offset) {
    debug('reinterpreting buffer to "%d" bytes', size);
    const rtn = exports._reinterpret(buffer, size, offset || 0);
    exports._attach(rtn, buffer);
    return rtn;
  };
  exports.reinterpretUntilZeros = function reinterpretUntilZeros(buffer, size, offset) {
    debug('reinterpreting buffer to until "%d" NULL (0) bytes are found', size);
    var rtn = exports._reinterpretUntilZeros(buffer, size, offset || 0);
    exports._attach(rtn, buffer);
    return rtn;
  };
  var types = exports.types = {};
  types.void = {
    size: 0,
    indirection: 1,
    get: function get(buf, offset) {
      debug("getting `void` type (returns `null`)");
      return null;
    },
    set: function set(buf, offset, val) {
      debug("setting `void` type (no-op)");
    }
  };
  types.int8 = {
    size: exports.sizeof.int8,
    indirection: 1,
    get: function get(buf, offset) {
      return buf.readInt8(offset || 0);
    },
    set: function set(buf, offset, val) {
      if (typeof val === "string") {
        val = val.charCodeAt(0);
      }
      return buf.writeInt8(val, offset || 0);
    }
  };
  types.uint8 = {
    size: exports.sizeof.uint8,
    indirection: 1,
    get: function get(buf, offset) {
      return buf.readUInt8(offset || 0);
    },
    set: function set(buf, offset, val) {
      if (typeof val === "string") {
        val = val.charCodeAt(0);
      }
      return buf.writeUInt8(val, offset || 0);
    }
  };
  types.int16 = {
    size: exports.sizeof.int16,
    indirection: 1,
    get: function get(buf, offset) {
      return buf["readInt16" + exports.endianness](offset || 0);
    },
    set: function set(buf, offset, val) {
      return buf["writeInt16" + exports.endianness](val, offset || 0);
    }
  };
  types.uint16 = {
    size: exports.sizeof.uint16,
    indirection: 1,
    get: function get(buf, offset) {
      return buf["readUInt16" + exports.endianness](offset || 0);
    },
    set: function set(buf, offset, val) {
      return buf["writeUInt16" + exports.endianness](val, offset || 0);
    }
  };
  types.int32 = {
    size: exports.sizeof.int32,
    indirection: 1,
    get: function get(buf, offset) {
      return buf["readInt32" + exports.endianness](offset || 0);
    },
    set: function set(buf, offset, val) {
      return buf["writeInt32" + exports.endianness](val, offset || 0);
    }
  };
  types.uint32 = {
    size: exports.sizeof.uint32,
    indirection: 1,
    get: function get(buf, offset) {
      return buf["readUInt32" + exports.endianness](offset || 0);
    },
    set: function set(buf, offset, val) {
      return buf["writeUInt32" + exports.endianness](val, offset || 0);
    }
  };
  types.int64 = {
    size: exports.sizeof.int64,
    indirection: 1,
    get: function get(buf, offset) {
      return buf["readInt64" + exports.endianness](offset || 0);
    },
    set: function set(buf, offset, val) {
      return buf["writeInt64" + exports.endianness](val, offset || 0);
    }
  };
  types.uint64 = {
    size: exports.sizeof.uint64,
    indirection: 1,
    get: function get(buf, offset) {
      return buf["readUInt64" + exports.endianness](offset || 0);
    },
    set: function set(buf, offset, val) {
      return buf["writeUInt64" + exports.endianness](val, offset || 0);
    }
  };
  types.float = {
    size: exports.sizeof.float,
    indirection: 1,
    get: function get(buf, offset) {
      return buf["readFloat" + exports.endianness](offset || 0);
    },
    set: function set(buf, offset, val) {
      return buf["writeFloat" + exports.endianness](val, offset || 0);
    }
  };
  types.double = {
    size: exports.sizeof.double,
    indirection: 1,
    get: function get(buf, offset) {
      return buf["readDouble" + exports.endianness](offset || 0);
    },
    set: function set(buf, offset, val) {
      return buf["writeDouble" + exports.endianness](val, offset || 0);
    }
  };
  types.Object = {
    size: exports.sizeof.Object,
    indirection: 1,
    get: function get(buf, offset) {
      return buf.readObject(offset || 0);
    },
    set: function set(buf, offset, val) {
      return buf.writeObject(val, offset || 0);
    }
  };
  types.CString = {
    size: exports.sizeof.pointer,
    alignment: exports.alignof.pointer,
    indirection: 1,
    get: function get(buf, offset) {
      const _buf = exports.readPointer(buf, offset);
      if (exports.isNull(_buf)) {
        return null;
      }
      return exports.readCString(_buf, 0);
    },
    set: function set(buf, offset, val) {
      let _buf;
      if (Buffer.isBuffer(val)) {
        _buf = val;
      } else {
        _buf = exports.allocCString(val);
      }
      return exports.writePointer(buf, offset, _buf);
    }
  };
  var utfstringwarned = false;
  Object.defineProperty(types, "Utf8String", {
    enumerable: false,
    configurable: true,
    get: function() {
      if (!utfstringwarned) {
        utfstringwarned = true;
        console.error('"Utf8String" type is deprecated, use "CString" instead');
      }
      return types.CString;
    }
  });
  [
    "bool",
    "byte",
    "char",
    "uchar",
    "short",
    "ushort",
    "int",
    "uint",
    "long",
    "ulong",
    "longlong",
    "ulonglong",
    "size_t"
  ].forEach((name) => {
    const unsigned = name === "bool" || name === "byte" || name === "size_t" || name[0] === "u";
    const size = exports.sizeof[name];
    assert(size >= 1 && size <= 8);
    let typeName = "int" + size * 8;
    if (unsigned) {
      typeName = "u" + typeName;
    }
    const type = exports.types[typeName];
    assert(type);
    exports.types[name] = Object.create(type);
  });
  Object.keys(exports.alignof).forEach((name) => {
    if (name === "pointer")
      return;
    exports.types[name].alignment = exports.alignof[name];
    assert(exports.types[name].alignment > 0);
  });
  exports.types.bool.get = function(_get) {
    return function get(buf, offset) {
      return _get(buf, offset) ? true : false;
    };
  }(exports.types.bool.get);
  exports.types.bool.set = function(_set) {
    return function set(buf, offset, val) {
      if (typeof val !== "number") {
        val = val ? 1 : 0;
      }
      return _set(buf, offset, val);
    };
  }(exports.types.bool.set);
  /*!
   * Set the `name` property of the types. Used for debugging...
   */
  Object.keys(exports.types).forEach((name) => {
    exports.types[name].name = name;
  });
  /*!
   * This `char *` type is used by "allocCString()" above.
   */
  var charPtrType = exports.refType(exports.types.char);
  /*!
   * Set the `type` property of the `NULL` pointer Buffer object.
   */
  exports.NULL.type = exports.types.void;
  exports.NULL_POINTER = exports.ref(exports.NULL);
  Buffer.prototype.address = function address() {
    return exports.address(this, 0);
  };
  Buffer.prototype.hexAddress = function hexAddress() {
    return exports.hexAddress(this, 0);
  };
  Buffer.prototype.isNull = function isNull() {
    return exports.isNull(this, 0);
  };
  Buffer.prototype.ref = function ref() {
    return exports.ref(this);
  };
  Buffer.prototype.deref = function deref() {
    return exports.deref(this);
  };
  Buffer.prototype.readObject = function readObject(offset) {
    return exports.readObject(this, offset);
  };
  Buffer.prototype.writeObject = function writeObject(obj, offset) {
    return exports.writeObject(this, offset, obj);
  };
  Buffer.prototype.readPointer = function readPointer(offset, size) {
    return exports.readPointer(this, offset, size);
  };
  Buffer.prototype.writePointer = function writePointer(ptr, offset) {
    return exports.writePointer(this, offset, ptr);
  };
  Buffer.prototype.readCString = function readCString(offset) {
    return exports.readCString(this, offset);
  };
  Buffer.prototype.writeCString = function writeCString(string, offset, encoding) {
    return exports.writeCString(this, offset, string, encoding);
  };
  Buffer.prototype.readInt64BE = function readInt64BE(offset) {
    return exports.readInt64BE(this, offset);
  };
  Buffer.prototype.writeInt64BE = function writeInt64BE(val, offset) {
    return exports.writeInt64BE(this, offset, val);
  };
  Buffer.prototype.readUInt64BE = function readUInt64BE(offset) {
    return exports.readUInt64BE(this, offset);
  };
  Buffer.prototype.writeUInt64BE = function writeUInt64BE(val, offset) {
    return exports.writeUInt64BE(this, offset, val);
  };
  Buffer.prototype.readInt64LE = function readInt64LE(offset) {
    return exports.readInt64LE(this, offset);
  };
  Buffer.prototype.writeInt64LE = function writeInt64LE(val, offset) {
    return exports.writeInt64LE(this, offset, val);
  };
  Buffer.prototype.readUInt64LE = function readUInt64LE(offset) {
    return exports.readUInt64LE(this, offset);
  };
  Buffer.prototype.writeUInt64LE = function writeUInt64LE(val, offset) {
    return exports.writeUInt64LE(this, offset, val);
  };
  Buffer.prototype.reinterpret = function reinterpret(size, offset) {
    return exports.reinterpret(this, size, offset);
  };
  Buffer.prototype.reinterpretUntilZeros = function reinterpretUntilZeros(size, offset) {
    return exports.reinterpretUntilZeros(this, size, offset);
  };
  var inspectSym = inspect.custom || "inspect";
  if (Buffer.prototype[inspectSym]) {
    Buffer.prototype[inspectSym] = overwriteInspect(Buffer.prototype[inspectSym]);
  }
  if (!(exports.NULL instanceof Buffer)) {
    debug("extending SlowBuffer's prototype since it doesn't inherit from Buffer.prototype");
    /*!
       * SlowBuffer convenience methods.
       */
    SlowBuffer = __require("buffer").SlowBuffer;
    SlowBuffer.prototype.address = Buffer.prototype.address;
    SlowBuffer.prototype.hexAddress = Buffer.prototype.hexAddress;
    SlowBuffer.prototype.isNull = Buffer.prototype.isNull;
    SlowBuffer.prototype.ref = Buffer.prototype.ref;
    SlowBuffer.prototype.deref = Buffer.prototype.deref;
    SlowBuffer.prototype.readObject = Buffer.prototype.readObject;
    SlowBuffer.prototype.writeObject = Buffer.prototype.writeObject;
    SlowBuffer.prototype.readPointer = Buffer.prototype.readPointer;
    SlowBuffer.prototype.writePointer = Buffer.prototype.writePointer;
    SlowBuffer.prototype.readCString = Buffer.prototype.readCString;
    SlowBuffer.prototype.writeCString = Buffer.prototype.writeCString;
    SlowBuffer.prototype.reinterpret = Buffer.prototype.reinterpret;
    SlowBuffer.prototype.reinterpretUntilZeros = Buffer.prototype.reinterpretUntilZeros;
    SlowBuffer.prototype.readInt64BE = Buffer.prototype.readInt64BE;
    SlowBuffer.prototype.writeInt64BE = Buffer.prototype.writeInt64BE;
    SlowBuffer.prototype.readUInt64BE = Buffer.prototype.readUInt64BE;
    SlowBuffer.prototype.writeUInt64BE = Buffer.prototype.writeUInt64BE;
    SlowBuffer.prototype.readInt64LE = Buffer.prototype.readInt64LE;
    SlowBuffer.prototype.writeInt64LE = Buffer.prototype.writeInt64LE;
    SlowBuffer.prototype.readUInt64LE = Buffer.prototype.readUInt64LE;
    SlowBuffer.prototype.writeUInt64LE = Buffer.prototype.writeUInt64LE;
    if (SlowBuffer.prototype[inspectSym]) {
      SlowBuffer.prototype[inspectSym] = overwriteInspect(SlowBuffer.prototype[inspectSym]);
    }
  }
  var SlowBuffer;
  function overwriteInspect(inspect2) {
    if (inspect2.name === "refinspect") {
      return inspect2;
    } else {
      return function refinspect() {
        var v = inspect2.apply(this, arguments);
        return v.replace("Buffer", "Buffer@0x" + this.hexAddress());
      };
    }
  }
});

// node_modules/ref-struct-di/node_modules/debug/src/common.js
var require_common2 = __commonJS((exports, module) => {
  function setup(env) {
    createDebug.debug = createDebug;
    createDebug.default = createDebug;
    createDebug.coerce = coerce;
    createDebug.disable = disable;
    createDebug.enable = enable;
    createDebug.enabled = enabled;
    createDebug.humanize = require_ms();
    Object.keys(env).forEach(function(key) {
      createDebug[key] = env[key];
    });
    createDebug.instances = [];
    createDebug.names = [];
    createDebug.skips = [];
    createDebug.formatters = {};
    function selectColor(namespace) {
      var hash = 0;
      for (var i = 0;i < namespace.length; i++) {
        hash = (hash << 5) - hash + namespace.charCodeAt(i);
        hash |= 0;
      }
      return createDebug.colors[Math.abs(hash) % createDebug.colors.length];
    }
    createDebug.selectColor = selectColor;
    function createDebug(namespace) {
      var prevTime;
      function debug() {
        if (!debug.enabled) {
          return;
        }
        for (var _len = arguments.length, args = new Array(_len), _key = 0;_key < _len; _key++) {
          args[_key] = arguments[_key];
        }
        var self = debug;
        var curr = Number(new Date);
        var ms = curr - (prevTime || curr);
        self.diff = ms;
        self.prev = prevTime;
        self.curr = curr;
        prevTime = curr;
        args[0] = createDebug.coerce(args[0]);
        if (typeof args[0] !== "string") {
          args.unshift("%O");
        }
        var index = 0;
        args[0] = args[0].replace(/%([a-zA-Z%])/g, function(match, format) {
          if (match === "%%") {
            return match;
          }
          index++;
          var formatter = createDebug.formatters[format];
          if (typeof formatter === "function") {
            var val = args[index];
            match = formatter.call(self, val);
            args.splice(index, 1);
            index--;
          }
          return match;
        });
        createDebug.formatArgs.call(self, args);
        var logFn = self.log || createDebug.log;
        logFn.apply(self, args);
      }
      debug.namespace = namespace;
      debug.enabled = createDebug.enabled(namespace);
      debug.useColors = createDebug.useColors();
      debug.color = selectColor(namespace);
      debug.destroy = destroy;
      debug.extend = extend;
      if (typeof createDebug.init === "function") {
        createDebug.init(debug);
      }
      createDebug.instances.push(debug);
      return debug;
    }
    function destroy() {
      var index = createDebug.instances.indexOf(this);
      if (index !== -1) {
        createDebug.instances.splice(index, 1);
        return true;
      }
      return false;
    }
    function extend(namespace, delimiter) {
      return createDebug(this.namespace + (typeof delimiter === "undefined" ? ":" : delimiter) + namespace);
    }
    function enable(namespaces) {
      createDebug.save(namespaces);
      createDebug.names = [];
      createDebug.skips = [];
      var i;
      var split = (typeof namespaces === "string" ? namespaces : "").split(/[\s,]+/);
      var len = split.length;
      for (i = 0;i < len; i++) {
        if (!split[i]) {
          continue;
        }
        namespaces = split[i].replace(/\*/g, ".*?");
        if (namespaces[0] === "-") {
          createDebug.skips.push(new RegExp("^" + namespaces.substr(1) + "$"));
        } else {
          createDebug.names.push(new RegExp("^" + namespaces + "$"));
        }
      }
      for (i = 0;i < createDebug.instances.length; i++) {
        var instance = createDebug.instances[i];
        instance.enabled = createDebug.enabled(instance.namespace);
      }
    }
    function disable() {
      createDebug.enable("");
    }
    function enabled(name) {
      if (name[name.length - 1] === "*") {
        return true;
      }
      var i;
      var len;
      for (i = 0, len = createDebug.skips.length;i < len; i++) {
        if (createDebug.skips[i].test(name)) {
          return false;
        }
      }
      for (i = 0, len = createDebug.names.length;i < len; i++) {
        if (createDebug.names[i].test(name)) {
          return true;
        }
      }
      return false;
    }
    function coerce(val) {
      if (val instanceof Error) {
        return val.stack || val.message;
      }
      return val;
    }
    createDebug.enable(createDebug.load());
    return createDebug;
  }
  module.exports = setup;
});

// node_modules/ref-struct-di/node_modules/debug/src/browser.js
var require_browser2 = __commonJS((exports, module) => {
  function _typeof(obj) {
    if (typeof Symbol === "function" && typeof Symbol.iterator === "symbol") {
      _typeof = function _typeof(obj2) {
        return typeof obj2;
      };
    } else {
      _typeof = function _typeof(obj2) {
        return obj2 && typeof Symbol === "function" && obj2.constructor === Symbol && obj2 !== Symbol.prototype ? "symbol" : typeof obj2;
      };
    }
    return _typeof(obj);
  }
  exports.log = log;
  exports.formatArgs = formatArgs;
  exports.save = save;
  exports.load = load;
  exports.useColors = useColors;
  exports.storage = localstorage();
  exports.colors = ["#0000CC", "#0000FF", "#0033CC", "#0033FF", "#0066CC", "#0066FF", "#0099CC", "#0099FF", "#00CC00", "#00CC33", "#00CC66", "#00CC99", "#00CCCC", "#00CCFF", "#3300CC", "#3300FF", "#3333CC", "#3333FF", "#3366CC", "#3366FF", "#3399CC", "#3399FF", "#33CC00", "#33CC33", "#33CC66", "#33CC99", "#33CCCC", "#33CCFF", "#6600CC", "#6600FF", "#6633CC", "#6633FF", "#66CC00", "#66CC33", "#9900CC", "#9900FF", "#9933CC", "#9933FF", "#99CC00", "#99CC33", "#CC0000", "#CC0033", "#CC0066", "#CC0099", "#CC00CC", "#CC00FF", "#CC3300", "#CC3333", "#CC3366", "#CC3399", "#CC33CC", "#CC33FF", "#CC6600", "#CC6633", "#CC9900", "#CC9933", "#CCCC00", "#CCCC33", "#FF0000", "#FF0033", "#FF0066", "#FF0099", "#FF00CC", "#FF00FF", "#FF3300", "#FF3333", "#FF3366", "#FF3399", "#FF33CC", "#FF33FF", "#FF6600", "#FF6633", "#FF9900", "#FF9933", "#FFCC00", "#FFCC33"];
  function useColors() {
    if (typeof window !== "undefined" && window.process && (window.process.type === "renderer" || window.process.__nwjs)) {
      return true;
    }
    if (typeof navigator !== "undefined" && navigator.userAgent && navigator.userAgent.toLowerCase().match(/(edge|trident)\/(\d+)/)) {
      return false;
    }
    return typeof document !== "undefined" && document.documentElement && document.documentElement.style && document.documentElement.style.WebkitAppearance || typeof window !== "undefined" && window.console && (window.console.firebug || window.console.exception && window.console.table) || typeof navigator !== "undefined" && navigator.userAgent && navigator.userAgent.toLowerCase().match(/firefox\/(\d+)/) && parseInt(RegExp.$1, 10) >= 31 || typeof navigator !== "undefined" && navigator.userAgent && navigator.userAgent.toLowerCase().match(/applewebkit\/(\d+)/);
  }
  function formatArgs(args) {
    args[0] = (this.useColors ? "%c" : "") + this.namespace + (this.useColors ? " %c" : " ") + args[0] + (this.useColors ? "%c " : " ") + "+" + module.exports.humanize(this.diff);
    if (!this.useColors) {
      return;
    }
    var c = "color: " + this.color;
    args.splice(1, 0, c, "color: inherit");
    var index = 0;
    var lastC = 0;
    args[0].replace(/%[a-zA-Z%]/g, function(match) {
      if (match === "%%") {
        return;
      }
      index++;
      if (match === "%c") {
        lastC = index;
      }
    });
    args.splice(lastC, 0, c);
  }
  function log() {
    var _console;
    return (typeof console === "undefined" ? "undefined" : _typeof(console)) === "object" && console.log && (_console = console).log.apply(_console, arguments);
  }
  function save(namespaces) {
    try {
      if (namespaces) {
        exports.storage.setItem("debug", namespaces);
      } else {
        exports.storage.removeItem("debug");
      }
    } catch (error) {}
  }
  function load() {
    var r;
    try {
      r = exports.storage.getItem("debug");
    } catch (error) {}
    if (!r && typeof process !== "undefined" && "env" in process) {
      r = process.env.DEBUG;
    }
    return r;
  }
  function localstorage() {
    try {
      return localStorage;
    } catch (error) {}
  }
  module.exports = require_common2()(exports);
  var formatters = module.exports.formatters;
  formatters.j = function(v) {
    try {
      return JSON.stringify(v);
    } catch (error) {
      return "[UnexpectedJSONParseError]: " + error.message;
    }
  };
});

// node_modules/ref-struct-di/node_modules/debug/src/node.js
var require_node2 = __commonJS((exports, module) => {
  var tty = __require("tty");
  var util = __require("util");
  exports.init = init;
  exports.log = log;
  exports.formatArgs = formatArgs;
  exports.save = save;
  exports.load = load;
  exports.useColors = useColors;
  exports.colors = [6, 2, 3, 4, 5, 1];
  try {
    supportsColor = (()=>{throw new Error("Cannot require module "+"supports-color");})();
    if (supportsColor && (supportsColor.stderr || supportsColor).level >= 2) {
      exports.colors = [20, 21, 26, 27, 32, 33, 38, 39, 40, 41, 42, 43, 44, 45, 56, 57, 62, 63, 68, 69, 74, 75, 76, 77, 78, 79, 80, 81, 92, 93, 98, 99, 112, 113, 128, 129, 134, 135, 148, 149, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 178, 179, 184, 185, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 214, 215, 220, 221];
    }
  } catch (error) {}
  var supportsColor;
  exports.inspectOpts = Object.keys(process.env).filter(function(key) {
    return /^debug_/i.test(key);
  }).reduce(function(obj, key) {
    var prop = key.substring(6).toLowerCase().replace(/_([a-z])/g, function(_, k) {
      return k.toUpperCase();
    });
    var val = process.env[key];
    if (/^(yes|on|true|enabled)$/i.test(val)) {
      val = true;
    } else if (/^(no|off|false|disabled)$/i.test(val)) {
      val = false;
    } else if (val === "null") {
      val = null;
    } else {
      val = Number(val);
    }
    obj[prop] = val;
    return obj;
  }, {});
  function useColors() {
    return "colors" in exports.inspectOpts ? Boolean(exports.inspectOpts.colors) : tty.isatty(process.stderr.fd);
  }
  function formatArgs(args) {
    var name = this.namespace, useColors2 = this.useColors;
    if (useColors2) {
      var c = this.color;
      var colorCode = "\x1B[3" + (c < 8 ? c : "8;5;" + c);
      var prefix = "  ".concat(colorCode, ";1m").concat(name, " \x1B[0m");
      args[0] = prefix + args[0].split(`
`).join(`
` + prefix);
      args.push(colorCode + "m+" + module.exports.humanize(this.diff) + "\x1B[0m");
    } else {
      args[0] = getDate() + name + " " + args[0];
    }
  }
  function getDate() {
    if (exports.inspectOpts.hideDate) {
      return "";
    }
    return new Date().toISOString() + " ";
  }
  function log() {
    return process.stderr.write(util.format.apply(util, arguments) + `
`);
  }
  function save(namespaces) {
    if (namespaces) {
      process.env.DEBUG = namespaces;
    } else {
      delete process.env.DEBUG;
    }
  }
  function load() {
    return process.env.DEBUG;
  }
  function init(debug) {
    debug.inspectOpts = {};
    var keys = Object.keys(exports.inspectOpts);
    for (var i = 0;i < keys.length; i++) {
      debug.inspectOpts[keys[i]] = exports.inspectOpts[keys[i]];
    }
  }
  module.exports = require_common2()(exports);
  var formatters = module.exports.formatters;
  formatters.o = function(v) {
    this.inspectOpts.colors = this.useColors;
    return util.inspect(v, this.inspectOpts).split(`
`).map(function(str) {
      return str.trim();
    }).join(" ");
  };
  formatters.O = function(v) {
    this.inspectOpts.colors = this.useColors;
    return util.inspect(v, this.inspectOpts);
  };
});

// node_modules/ref-struct-di/node_modules/debug/src/index.js
var require_src2 = __commonJS((exports, module) => {
  if (typeof process === "undefined" || process.type === "renderer" || false || process.__nwjs) {
    module.exports = require_browser2();
  } else {
    module.exports = require_node2();
  }
});

// node_modules/ref-struct-di/lib/struct.js
var require_struct = __commonJS((exports, module) => {
  var util = __require("util");
  var assert = __require("assert");
  var debug = require_src2()("ref:struct");
  module.exports = function(ref) {
    function Struct() {
      debug('defining new struct "type"');
      function StructType(arg2, data) {
        if (!(this instanceof StructType)) {
          return new StructType(arg2, data);
        }
        debug("creating new struct instance");
        var store;
        if (Buffer.isBuffer(arg2)) {
          debug("using passed-in Buffer instance to back the struct", arg2);
          assert(arg2.length >= StructType.size, "Buffer instance must be at least " + StructType.size + " bytes to back this struct type");
          store = arg2;
          arg2 = data;
        } else {
          debug("creating new Buffer instance to back the struct (size: %d)", StructType.size);
          store = Buffer.alloc(StructType.size);
        }
        store.type = StructType;
        this["ref.buffer"] = store;
        if (arg2) {
          for (var key in arg2) {
            this[key] = arg2[key];
          }
        }
        StructType._instanceCreated = true;
      }
      StructType.prototype = Object.create(proto, {
        constructor: {
          value: StructType,
          enumerable: false,
          writable: true,
          configurable: true
        }
      });
      StructType.defineProperty = defineProperty;
      StructType.toString = toString;
      StructType.fields = {};
      var opt = arguments.length > 0 && arguments[1] ? arguments[1] : {};
      StructType.size = 0;
      StructType.alignment = 0;
      StructType.indirection = 1;
      StructType.isPacked = opt.packed ? Boolean(opt.packed) : false;
      StructType.get = get;
      StructType.set = set;
      var arg = arguments[0];
      if (Array.isArray(arg)) {
        arg.forEach(function(a) {
          var type = a[0];
          var name = a[1];
          StructType.defineProperty(name, type);
        });
      } else if (typeof arg === "object") {
        Object.keys(arg).forEach(function(name) {
          var type = arg[name];
          StructType.defineProperty(name, type);
        });
      }
      return StructType;
    }
    function get(buffer, offset) {
      debug('Struct "type" getter for buffer at offset', buffer, offset);
      if (offset > 0) {
        buffer = buffer.slice(offset);
      }
      return new this(buffer);
    }
    function set(buffer, offset, value) {
      debug('Struct "type" setter for buffer at offset', buffer, offset, value);
      var isStruct = value instanceof this;
      if (isStruct) {
        value["ref.buffer"].copy(buffer, offset, 0, this.size);
      } else {
        if (offset > 0) {
          buffer = buffer.slice(offset);
        }
        new this(buffer, value);
      }
    }
    function toString() {
      return "[StructType]";
    }
    function defineProperty(name, type) {
      debug("defining new struct type field", name);
      type = ref.coerceType(type);
      assert(!this._instanceCreated, "an instance of this Struct type has already " + 'been created, cannot add new "fields" anymore');
      assert.equal("string", typeof name, 'expected a "string" field name');
      assert(type && /object|function/i.test(typeof type) && "size" in type && "indirection" in type, 'expected a "type" object describing the field type: "' + type + '"');
      assert(type.indirection > 1 || type.size > 0, '"type" object must have a size greater than 0');
      assert(!(name in this.prototype), 'the field "' + name + '" already exists in this Struct type');
      var field = {
        type
      };
      this.fields[name] = field;
      var desc = { enumerable: true, configurable: true };
      desc.get = function() {
        debug('getting "%s" struct field (offset: %d)', name, field.offset);
        return ref.get(this["ref.buffer"], field.offset, type);
      };
      desc.set = function(value) {
        debug('setting "%s" struct field (offset: %d)', name, field.offset, value);
        return ref.set(this["ref.buffer"], field.offset, value, type);
      };
      recalc(this);
      Object.defineProperty(this.prototype, name, desc);
    }
    function recalc(struct) {
      struct.size = 0;
      struct.alignment = 0;
      var fieldNames = Object.keys(struct.fields);
      fieldNames.forEach(function(name) {
        var field = struct.fields[name];
        var type = field.type;
        var alignment = type.alignment || ref.alignof.pointer;
        if (type.indirection > 1) {
          alignment = ref.alignof.pointer;
        }
        if (struct.isPacked) {
          struct.alignment = Math.min(struct.alignment || alignment, alignment);
        } else {
          struct.alignment = Math.max(struct.alignment, alignment);
        }
      });
      fieldNames.forEach(function(name) {
        var field = struct.fields[name];
        var type = field.type;
        if (type.fixedLength != null) {
          field.offset = addType(type.type);
          for (var i = 1;i < type.fixedLength; i++) {
            addType(type.type);
          }
        } else {
          field.offset = addType(type);
        }
      });
      function addType(type) {
        var offset = struct.size;
        var align = type.indirection === 1 ? type.alignment : ref.alignof.pointer;
        var padding = struct.isPacked ? 0 : (align - offset % align) % align;
        var size = type.indirection === 1 ? type.size : ref.sizeof.pointer;
        offset += padding;
        if (!struct.isPacked) {
          assert.equal(offset % align, 0, "offset should align");
        }
        struct.size = offset + size;
        return offset;
      }
      var left = struct.size % struct.alignment;
      if (left > 0) {
        debug("additional padding to the end of struct:", struct.alignment - left);
        struct.size += struct.alignment - left;
      }
    }
    var proto = {};
    proto["ref.buffer"] = ref.NULL;
    proto.toObject = function toObject() {
      var obj = {};
      Object.keys(this.constructor.fields).forEach(function(k) {
        obj[k] = this[k];
      }, this);
      return obj;
    };
    proto.toJSON = function toJSON() {
      return this.toObject();
    };
    proto.inspect = function inspect() {
      var obj = this.toObject();
      Object.keys(this).forEach(function(k) {
        obj[k] = this[k];
      }, this);
      return util.inspect(obj);
    };
    proto.ref = function ref() {
      return this["ref.buffer"];
    };
    return Struct;
  };
});

// node_modules/ffi-napi/lib/bindings.js
var require_bindings = __commonJS((exports, module) => {
  var __dirname = "/home/calc/Documents/keylite/bindings/js/node_modules/ffi-napi/lib";
  var path = __require("path");
  var ref = require_ref();
  var assert = __require("assert");
  assert(ref.instance);
  var bindings = require_node_gyp_build2()(path.join(__dirname, ".."));
  module.exports = bindings.initializeBindings(ref.instance);
});

// node_modules/ffi-napi/lib/type.js
var require_type = __commonJS((exports, module) => {
  var ref = require_ref();
  var assert = __require("assert");
  var debug = require_src()("ffi:types");
  var Struct = require_struct()(ref);
  var bindings = require_bindings();
  var FFI_TYPE = Type.FFI_TYPE = Struct();
  FFI_TYPE.defineProperty("size", ref.types.size_t);
  FFI_TYPE.defineProperty("alignment", ref.types.ushort);
  FFI_TYPE.defineProperty("type", ref.types.ushort);
  var ffi_type_ptr_array = ref.refType(ref.refType(FFI_TYPE));
  FFI_TYPE.defineProperty("elements", ffi_type_ptr_array);
  assert.strictEqual(bindings.FFI_TYPE_SIZE, FFI_TYPE.size);
  function Type(type) {
    type = ref.coerceType(type);
    debug("Type()", type.name || type);
    assert(type.indirection >= 1, 'invalid "type" given: ' + (type.name || type));
    let ret;
    if (type.indirection === 1) {
      ret = type.ffi_type;
    } else {
      ret = bindings.FFI_TYPES.pointer;
    }
    if (!ret && type.type) {
      ret = bindings.FFI_TYPES.pointer;
    }
    if (!ret && type.fields) {
      debug('creating an `ffi_type` for given "ref-struct" type');
      const fields = type.fields;
      const fieldNames = Object.keys(fields);
      const numFields = fieldNames.length;
      let numElements = 0;
      const ffi_type = new FFI_TYPE;
      let field;
      let ffi_type_ptr;
      ffi_type.size = 0;
      ffi_type.alignment = 0;
      ffi_type.type = 13;
      for (let i = 0;i < numFields; i++) {
        field = fields[fieldNames[i]];
        if (field.type.fixedLength > 0) {
          numElements += field.type.fixedLength;
        } else {
          numElements += 1;
        }
      }
      const size = ref.sizeof.pointer * (numElements + 1);
      const elements = ffi_type.elements = Buffer.alloc(size);
      let index = 0;
      for (let i = 0;i < numFields; i++) {
        field = fields[fieldNames[i]];
        if (field.type.fixedLength > 0) {
          ffi_type_ptr = Type(field.type.type);
          for (var j = 0;j < field.type.fixedLength; j++) {
            elements.writePointer(ffi_type_ptr, index++ * ref.sizeof.pointer);
          }
        } else {
          ffi_type_ptr = Type(field.type);
          elements.writePointer(ffi_type_ptr, index++ * ref.sizeof.pointer);
        }
      }
      elements.writePointer(ref.NULL, index * ref.sizeof.pointer);
      ret = type.ffi_type = ffi_type.ref();
    }
    if (!ret && type.name) {
      if (type.name == "CString") {
        ret = type.ffi_type = bindings.FFI_TYPES.pointer;
      } else {
        let cur = type;
        while (!ret && cur) {
          ret = cur.ffi_type = bindings.FFI_TYPES[cur.name];
          cur = Object.getPrototypeOf(cur);
        }
      }
    }
    assert(ret, "Could not determine the `ffi_type` instance for type: " + (type.name || type));
    debug("returning `ffi_type`", ret.name);
    return ret;
  }
  module.exports = Type;
});

// node_modules/ffi-napi/lib/cif.js
var require_cif = __commonJS((exports, module) => {
  var Type = require_type();
  var assert = __require("assert");
  var debug = require_src()("ffi:cif");
  var ref = require_ref();
  var bindings = require_bindings();
  var POINTER_SIZE = ref.sizeof.pointer;
  var ffi_prep_cif = bindings.ffi_prep_cif;
  var FFI_CIF_SIZE = bindings.FFI_CIF_SIZE;
  var FFI_DEFAULT_ABI = bindings.FFI_DEFAULT_ABI;
  var FFI_OK = bindings.FFI_OK;
  var FFI_BAD_TYPEDEF = bindings.FFI_BAD_TYPEDEF;
  var FFI_BAD_ABI = bindings.FFI_BAD_ABI;
  var cifs = [];
  function CIF(rtype, types, abi) {
    debug("creating `ffi_cif *` instance");
    assert(!!rtype, 'expected a return "type" object as the first argument');
    assert(Array.isArray(types), 'expected an Array of arg "type" objects as the second argument');
    const cif = Buffer.alloc(FFI_CIF_SIZE);
    const numArgs = types.length;
    const _argtypesptr = Buffer.alloc(numArgs * POINTER_SIZE);
    const _rtypeptr = Type(rtype);
    for (var i = 0;i < numArgs; i++) {
      const type = types[i];
      const ffiType = Type(type);
      _argtypesptr.writePointer(ffiType, i * POINTER_SIZE);
    }
    cif.rtnTypePtr = _rtypeptr;
    cif.argTypesPtr = _argtypesptr;
    if (typeof abi === "undefined") {
      debug("no ABI specified (this is OK), using FFI_DEFAULT_ABI");
      abi = FFI_DEFAULT_ABI;
    }
    const status = ffi_prep_cif(cif, numArgs, _rtypeptr, _argtypesptr, abi);
    if (status !== FFI_OK) {
      switch (status) {
        case FFI_BAD_TYPEDEF: {
          const err = new Error("ffi_prep_cif() returned an FFI_BAD_TYPEDEF error");
          err.code = "FFI_BAD_TYPEDEF";
          err.errno = status;
          throw err;
        }
        case FFI_BAD_ABI: {
          const err = new Error("ffi_prep_cif() returned an FFI_BAD_ABI error");
          err.code = "FFI_BAD_ABI";
          err.errno = status;
          throw err;
        }
        default:
          throw new Error("ffi_prep_cif() returned an error: " + status);
      }
    }
    if (debug.enabled || `${process.env.DEBUG}`.match(/\bffi\b/))
      cifs.push(cif);
    return cif;
  }
  module.exports = CIF;
});

// node_modules/ffi-napi/lib/cif_var.js
var require_cif_var = __commonJS((exports, module) => {
  var Type = require_type();
  var assert = __require("assert");
  var debug = require_src()("ffi:cif_var");
  var ref = require_ref();
  var bindings = require_bindings();
  var POINTER_SIZE = ref.sizeof.pointer;
  var ffi_prep_cif_var = bindings.ffi_prep_cif_var;
  var FFI_CIF_SIZE = bindings.FFI_CIF_SIZE;
  var FFI_DEFAULT_ABI = bindings.FFI_DEFAULT_ABI;
  var FFI_OK = bindings.FFI_OK;
  var FFI_BAD_TYPEDEF = bindings.FFI_BAD_TYPEDEF;
  var FFI_BAD_ABI = bindings.FFI_BAD_ABI;
  function CIF_var(rtype, types, numFixedArgs, abi) {
    debug("creating `ffi_cif *` instance with `ffi_prep_cif_var()`");
    assert(!!rtype, 'expected a return "type" object as the first argument');
    assert(Array.isArray(types), 'expected an Array of arg "type" objects as the second argument');
    assert(numFixedArgs >= 1, "expected the number of fixed arguments to be at least 1");
    const cif = Buffer.alloc(FFI_CIF_SIZE);
    const numTotalArgs = types.length;
    const _argtypesptr = Buffer.alloc(numTotalArgs * POINTER_SIZE);
    const _rtypeptr = Type(rtype);
    for (let i = 0;i < numTotalArgs; i++) {
      const ffiType = Type(types[i]);
      _argtypesptr.writePointer(ffiType, i * POINTER_SIZE);
    }
    cif.rtnTypePtr = _rtypeptr;
    cif.argTypesPtr = _argtypesptr;
    if (typeof abi === "undefined") {
      debug("no ABI specified (this is OK), using FFI_DEFAULT_ABI");
      abi = FFI_DEFAULT_ABI;
    }
    const status = ffi_prep_cif_var(cif, numFixedArgs, numTotalArgs, _rtypeptr, _argtypesptr, abi);
    if (status !== FFI_OK) {
      switch (status) {
        case FFI_BAD_TYPEDEF: {
          const err = new Error("ffi_prep_cif_var() returned an FFI_BAD_TYPEDEF error");
          err.code = "FFI_BAD_TYPEDEF";
          err.errno = status;
          throw err;
        }
        case FFI_BAD_ABI: {
          const err = new Error("ffi_prep_cif_var() returned an FFI_BAD_ABI error");
          err.code = "FFI_BAD_ABI";
          err.errno = status;
          throw err;
        }
        default: {
          const err = new Error("ffi_prep_cif_var() returned an error: " + status);
          err.errno = status;
          throw err;
        }
      }
    }
    return cif;
  }
  module.exports = CIF_var;
});

// node_modules/ffi-napi/lib/callback.js
var require_callback = __commonJS((exports, module) => {
  var ref = require_ref();
  var CIF = require_cif();
  var assert = __require("assert");
  var debug = require_src()("ffi:Callback");
  var _Callback = require_bindings().Callback;
  function errorReportCallback(err) {
    if (err) {
      process.nextTick(function() {
        if (typeof err === "string") {
          throw new Error(err);
        } else {
          throw err;
        }
      });
    }
  }
  function Callback(retType, argTypes, abi, func) {
    debug("creating new Callback");
    if (typeof abi === "function") {
      func = abi;
      abi = undefined;
    }
    assert(!!retType, 'expected a return "type" object as the first argument');
    assert(Array.isArray(argTypes), 'expected Array of arg "type" objects as the second argument');
    assert.equal(typeof func, "function", "expected a function as the third argument");
    retType = ref.coerceType(retType);
    argTypes = argTypes.map(ref.coerceType);
    const cif = CIF(retType, argTypes, abi);
    const argc = argTypes.length;
    const callback = _Callback(cif, retType.size, argc, errorReportCallback, (retval, params) => {
      debug("Callback function being invoked");
      try {
        const args = [];
        for (var i = 0;i < argc; i++) {
          const type = argTypes[i];
          const argPtr = params.readPointer(i * ref.sizeof.pointer, type.size);
          argPtr.type = type;
          args.push(argPtr.deref());
        }
        const result = func.apply(null, args);
        try {
          ref.set(retval, 0, result, retType);
        } catch (e) {
          e.message = "error setting return value - " + e.message;
          throw e;
        }
      } catch (e) {
        return e;
      }
    });
    callback._cif = cif;
    return callback;
  }
  module.exports = Callback;
});

// node_modules/ffi-napi/lib/_foreign_function.js
var require__foreign_function = __commonJS((exports, module) => {
  var assert = __require("assert");
  var debug = require_src()("ffi:_ForeignFunction");
  var ref = require_ref();
  var bindings = require_bindings();
  var POINTER_SIZE = ref.sizeof.pointer;
  var FFI_ARG_SIZE = bindings.FFI_ARG_SIZE;
  function ForeignFunction(cif, funcPtr, returnType, argTypes) {
    debug("creating new ForeignFunction", funcPtr);
    const numArgs = argTypes.length;
    const argsArraySize = numArgs * POINTER_SIZE;
    const resultSize = returnType.size >= ref.sizeof.long ? returnType.size : FFI_ARG_SIZE;
    assert(resultSize > 0);
    const proxy = function() {
      debug("invoking proxy function");
      if (arguments.length !== numArgs) {
        throw new TypeError("Expected " + numArgs + " arguments, got " + arguments.length);
      }
      const result = Buffer.alloc(resultSize);
      const argsList = Buffer.alloc(argsArraySize);
      let i;
      try {
        for (i = 0;i < numArgs; i++) {
          const argType = argTypes[i];
          const val = arguments[i];
          const valPtr = ref.alloc(argType, val);
          argsList.writePointer(valPtr, i * POINTER_SIZE);
        }
      } catch (e) {
        i++;
        e.message = "error setting argument " + i + " - " + e.message;
        throw e;
      }
      bindings.ffi_call(cif, funcPtr, result, argsList);
      result.type = returnType;
      return result.deref();
    };
    proxy.async = function() {
      debug("invoking async proxy function");
      const argc = arguments.length;
      if (argc !== numArgs + 1) {
        throw new TypeError("Expected " + (numArgs + 1) + " arguments, got " + argc);
      }
      const callback = arguments[argc - 1];
      if (typeof callback !== "function") {
        throw new TypeError("Expected a callback function as argument number: " + (argc - 1));
      }
      const result = Buffer.alloc(resultSize);
      const argsList = Buffer.alloc(argsArraySize);
      let i;
      try {
        for (i = 0;i < numArgs; i++) {
          const argType = argTypes[i];
          const val = arguments[i];
          const valPtr = ref.alloc(argType, val);
          argsList.writePointer(valPtr, i * POINTER_SIZE);
        }
      } catch (e) {
        e.message = "error setting argument " + i + " - " + e.message;
        return process.nextTick(callback.bind(null, e));
      }
      bindings.ffi_call_async(cif, funcPtr, result, argsList, function(err) {
        [cif, funcPtr, argsList].map(() => {});
        if (err) {
          callback(err);
        } else {
          result.type = returnType;
          callback(null, result.deref());
        }
      });
    };
    return proxy;
  }
  module.exports = ForeignFunction;
});

// node_modules/ffi-napi/lib/foreign_function.js
var require_foreign_function = __commonJS((exports, module) => {
  var CIF = require_cif();
  var _ForeignFunction = require__foreign_function();
  var debug = require_src()("ffi:ForeignFunction");
  var assert = __require("assert");
  var ref = require_ref();
  function ForeignFunction(funcPtr, returnType, argTypes, abi) {
    debug("creating new ForeignFunction", funcPtr);
    assert(Buffer.isBuffer(funcPtr), "expected Buffer as first argument");
    assert(!!returnType, 'expected a return "type" object as the second argument');
    assert(Array.isArray(argTypes), 'expected Array of arg "type" objects as the third argument');
    returnType = ref.coerceType(returnType);
    argTypes = argTypes.map(ref.coerceType);
    const cif = CIF(returnType, argTypes, abi);
    return _ForeignFunction(cif, funcPtr, returnType, argTypes);
  }
  module.exports = ForeignFunction;
});

// node_modules/ffi-napi/lib/function.js
var require_function = __commonJS((exports, module) => {
  var ref = require_ref();
  var assert = __require("assert");
  var bindings = require_bindings();
  var Callback = require_callback();
  var ForeignFunction = require_foreign_function();
  var debug = require_src()("ffi:FunctionType");
  module.exports = Function;
  function Function(retType, argTypes, abi) {
    if (!(this instanceof Function)) {
      return new Function(retType, argTypes, abi);
    }
    debug("creating new FunctionType");
    assert(!!retType, 'expected a return "type" object as the first argument');
    assert(Array.isArray(argTypes), 'expected Array of arg "type" objects as the second argument');
    this.retType = ref.coerceType(retType);
    this.argTypes = argTypes.map(ref.coerceType);
    this.abi = abi == null ? bindings.FFI_DEFAULT_ABI : abi;
  }
  Function.prototype.ffi_type = bindings.FFI_TYPES.pointer;
  Function.prototype.size = ref.sizeof.pointer;
  Function.prototype.alignment = ref.alignof.pointer;
  Function.prototype.indirection = 1;
  Function.prototype.toPointer = function toPointer(fn) {
    return Callback(this.retType, this.argTypes, this.abi, fn);
  };
  Function.prototype.toFunction = function toFunction(buf) {
    return ForeignFunction(buf, this.retType, this.argTypes, this.abi);
  };
  Function.prototype.get = function get(buffer, offset) {
    debug('ffi FunctionType "get" function');
    const ptr = buffer.readPointer(offset);
    return this.toFunction(ptr);
  };
  Function.prototype.set = function set(buffer, offset, value) {
    debug('ffi FunctionType "set" function');
    let ptr;
    if (typeof value == "function") {
      ptr = this.toPointer(value);
    } else if (Buffer.isBuffer(value)) {
      ptr = value;
    } else {
      throw new Error("don't know how to set callback function for: " + value);
    }
    buffer.writePointer(ptr, offset);
  };
});

// node_modules/ffi-napi/lib/foreign_function_var.js
var require_foreign_function_var = __commonJS((exports, module) => {
  var CIF_var = require_cif_var();
  var Type = require_type();
  var _ForeignFunction = require__foreign_function();
  var assert = __require("assert");
  var debug = require_src()("ffi:VariadicForeignFunction");
  var ref = require_ref();
  var bindings = require_bindings();
  var POINTER_SIZE = ref.sizeof.pointer;
  var FFI_ARG_SIZE = bindings.FFI_ARG_SIZE;
  function VariadicForeignFunction(funcPtr, returnType, fixedArgTypes, abi) {
    debug("creating new VariadicForeignFunction", funcPtr);
    const cache = {};
    assert(Buffer.isBuffer(funcPtr), "expected Buffer as first argument");
    assert(!!returnType, 'expected a return "type" object as the second argument');
    assert(Array.isArray(fixedArgTypes), 'expected Array of arg "type" objects as the third argument');
    const numFixedArgs = fixedArgTypes.length;
    fixedArgTypes = fixedArgTypes.map(ref.coerceType);
    const fixedKey = fixedArgTypes.map(function(type) {
      return getId(type);
    });
    function variadic_function_generator() {
      debug("variadic_function_generator invoked");
      const argTypes = fixedArgTypes.slice();
      let key = fixedKey.slice();
      for (let i = 0;i < arguments.length; i++) {
        const type = ref.coerceType(arguments[i]);
        argTypes.push(type);
        const ffi_type = Type(type);
        assert(ffi_type.name);
        key.push(getId(type));
      }
      const rtnType = ref.coerceType(variadic_function_generator.returnType);
      const rtnName = getId(rtnType);
      assert(rtnName);
      key = rtnName + key.join("");
      let func = cache[key];
      if (func) {
        debug("cache hit for key:", key);
      } else {
        debug("creating the variadic ffi_cif instance for key:", key);
        const cif = CIF_var(returnType, argTypes, numFixedArgs, abi);
        func = cache[key] = _ForeignFunction(cif, funcPtr, rtnType, argTypes);
      }
      return func;
    }
    variadic_function_generator.returnType = returnType;
    return variadic_function_generator;
  }
  module.exports = VariadicForeignFunction;
  var idKey = "_ffiId";
  var counter = 0;
  function getId(type) {
    if (!type.hasOwnProperty(idKey)) {
      type[idKey] = (counter++ * 65536 | 0).toString(16);
    }
    return type[idKey];
  }
});

// node_modules/ffi-napi/lib/dynamic_library.js
var require_dynamic_library = __commonJS((exports, module) => {
  var ForeignFunction = require_foreign_function();
  var assert = __require("assert");
  var debug = require_src()("ffi:DynamicLibrary");
  var bindings = require_bindings();
  var funcs = bindings.StaticFunctions;
  var ref = require_ref();
  var read = __require("fs").readFileSync;
  var int = ref.types.int;
  var voidPtr = ref.refType(ref.types.void);
  var dlopen = ForeignFunction(funcs.dlopen, voidPtr, ["string", int]);
  var dlclose = ForeignFunction(funcs.dlclose, int, [voidPtr]);
  var dlsym = ForeignFunction(funcs.dlsym, voidPtr, [voidPtr, "string"]);
  var dlerror = ForeignFunction(funcs.dlerror, "string", []);
  function DynamicLibrary(path, mode) {
    if (!(this instanceof DynamicLibrary)) {
      return new DynamicLibrary(path, mode);
    }
    debug("new DynamicLibrary()", path, mode);
    if (mode == null) {
      mode = DynamicLibrary.FLAGS.RTLD_LAZY;
    }
    this._path = path;
    this._handle = dlopen(path, mode);
    assert(Buffer.isBuffer(this._handle), "expected a Buffer instance to be returned from `dlopen()`");
    if (this._handle.isNull()) {
      var err = this.error();
      let match;
      if (match = err.match(/^(([^ \t()])+\.so([^ \t:()])*):([ \t])*/)) {
        const content = read(match[1], "ascii");
        if (match = content.match(/GROUP *\( *(([^ )])+)/)) {
          return DynamicLibrary.call(this, match[1], mode);
        }
      }
      throw new Error("Dynamic Linking Error: " + err);
    }
  }
  module.exports = DynamicLibrary;
  DynamicLibrary.FLAGS = {};
  Object.keys(bindings).forEach(function(k) {
    if (!/^RTLD_/.test(k))
      return;
    const desc = Object.getOwnPropertyDescriptor(bindings, k);
    Object.defineProperty(DynamicLibrary.FLAGS, k, desc);
  });
  DynamicLibrary.prototype.close = function() {
    debug("dlclose()");
    return dlclose(this._handle);
  };
  DynamicLibrary.prototype.get = function(symbol) {
    debug("dlsym()", symbol);
    assert.strictEqual("string", typeof symbol);
    const ptr = dlsym(this._handle, symbol);
    assert(Buffer.isBuffer(ptr));
    if (ptr.isNull()) {
      throw new Error("Dynamic Symbol Retrieval Error: " + this.error());
    }
    ptr.name = symbol;
    return ptr;
  };
  DynamicLibrary.prototype.error = function error() {
    debug("dlerror()");
    return dlerror();
  };
  DynamicLibrary.prototype.path = function error() {
    return this._path;
  };
});

// node_modules/ffi-napi/lib/library.js
var require_library = __commonJS((exports, module) => {
  var DynamicLibrary = require_dynamic_library();
  var ForeignFunction = require_foreign_function();
  var VariadicForeignFunction = require_foreign_function_var();
  var debug = require_src()("ffi:Library");
  var RTLD_NOW = DynamicLibrary.FLAGS.RTLD_NOW;
  var EXT = Library.EXT = {
    linux: ".so",
    linux2: ".so",
    sunos: ".so",
    solaris: ".so",
    freebsd: ".so",
    openbsd: ".so",
    darwin: ".dylib",
    mac: ".dylib",
    win32: ".dll"
  }[process.platform];
  function Library(libfile, funcs, lib) {
    debug("creating Library object for", libfile);
    if (libfile && typeof libfile === "string" && libfile.indexOf(EXT) === -1) {
      debug("appending library extension to library name", EXT);
      libfile += EXT;
    }
    if (!lib) {
      lib = {};
    }
    let dl;
    if (typeof libfile === "string" || !libfile) {
      dl = new DynamicLibrary(libfile || null, RTLD_NOW);
    } else {
      dl = libfile;
    }
    Object.keys(funcs || {}).forEach(function(func) {
      debug("defining function", func);
      const fptr = dl.get(func);
      const info = funcs[func];
      if (fptr.isNull()) {
        throw new Error('Library: "' + dl.path() + '" returned NULL function pointer for "' + func + '"');
      }
      const resultType = info[0];
      const paramTypes = info[1];
      const fopts = info[2];
      const abi = fopts && fopts.abi;
      const async = fopts && fopts.async;
      const varargs = fopts && fopts.varargs;
      if (varargs) {
        lib[func] = VariadicForeignFunction(fptr, resultType, paramTypes, abi);
      } else {
        const ff = ForeignFunction(fptr, resultType, paramTypes, abi);
        lib[func] = async ? ff.async : ff;
      }
    });
    return lib;
  }
  module.exports = Library;
});

// node_modules/ffi-napi/lib/errno.js
var require_errno = __commonJS((exports, module) => {
  var DynamicLibrary = require_dynamic_library();
  var ForeignFunction = require_foreign_function();
  var bindings = require_bindings();
  var funcs = bindings.StaticFunctions;
  var ref = require_ref();
  var int = ref.types.int;
  var intPtr = ref.refType(int);
  var errno = null;
  if (process.platform == "win32") {
    const _errno = DynamicLibrary("msvcrt.dll").get("_errno");
    const errnoPtr = ForeignFunction(_errno, intPtr, []);
    errno = function() {
      return errnoPtr().deref();
    };
  } else {
    errno = ForeignFunction(funcs._errno, "int", []);
  }
  module.exports = errno;
});

// node_modules/ffi-napi/lib/ffi.js
var require_ffi = __commonJS((exports) => {
  var ref = require_ref();
  var assert = __require("assert");
  var debug = require_src()("ffi:ffi");
  var Struct = require_struct()(ref);
  var bindings = require_bindings();
  [
    "FFI_TYPES",
    "FFI_OK",
    "FFI_BAD_TYPEDEF",
    "FFI_BAD_ABI",
    "FFI_DEFAULT_ABI",
    "FFI_FIRST_ABI",
    "FFI_LAST_ABI",
    "FFI_SYSV",
    "FFI_UNIX64",
    "FFI_WIN64",
    "FFI_VFP",
    "FFI_STDCALL",
    "FFI_THISCALL",
    "FFI_FASTCALL",
    "RTLD_LAZY",
    "RTLD_NOW",
    "RTLD_LOCAL",
    "RTLD_GLOBAL",
    "RTLD_NOLOAD",
    "RTLD_NODELETE",
    "RTLD_FIRST",
    "RTLD_NEXT",
    "RTLD_DEFAULT",
    "RTLD_SELF",
    "RTLD_MAIN_ONLY",
    "FFI_MS_CDECL"
  ].forEach((prop) => {
    if (!bindings.hasOwnProperty(prop)) {
      return debug("skipping exporting of non-existant property", prop);
    }
    const desc = Object.getOwnPropertyDescriptor(bindings, prop);
    Object.defineProperty(exports, prop, desc);
  });
  Object.keys(bindings.FFI_TYPES).forEach((name) => {
    const type = bindings.FFI_TYPES[name];
    type.name = name;
    if (name === "pointer")
      return;
    ref.types[name].ffi_type = type;
  });
  ref.types.size_t.ffi_type = bindings.FFI_TYPES.pointer;
  var CString = ref.types.CString || ref.types.Utf8String;
  CString.ffi_type = bindings.FFI_TYPES.pointer;
  ref.types.Object.ffi_type = bindings.FFI_TYPES.pointer;
  switch (ref.sizeof.long) {
    case 4:
      ref.types.ulong.ffi_type = bindings.FFI_TYPES.uint32;
      ref.types.long.ffi_type = bindings.FFI_TYPES.int32;
      break;
    case 8:
      ref.types.ulong.ffi_type = bindings.FFI_TYPES.uint64;
      ref.types.long.ffi_type = bindings.FFI_TYPES.int64;
      break;
    default:
      throw new Error('unsupported "long" size: ' + ref.sizeof.long);
  }
  exports.types = ref.types;
  exports.version = bindings.version;
  exports.CIF = require_cif();
  exports.CIF_var = require_cif_var();
  exports.Function = require_function();
  exports.ForeignFunction = require_foreign_function();
  exports.VariadicForeignFunction = require_foreign_function_var();
  exports.DynamicLibrary = require_dynamic_library();
  exports.Library = require_library();
  exports.Callback = require_callback();
  exports.errno = require_errno();
  exports.ffiType = require_type();
  exports.LIB_EXT = exports.Library.EXT;
  exports.FFI_TYPE = exports.ffiType.FFI_TYPE;
});

// keylite-ffi.ts
var import_ffi_napi = __toESM(require_ffi(), 1);
var import_ref_napi = __toESM(require_ref(), 1);
var voidPtr = import_ref_napi.default.refType(import_ref_napi.default.types.void);
var voidPtrPtr = import_ref_napi.default.refType(voidPtr);
var ucharPtr = import_ref_napi.default.refType(import_ref_napi.default.types.uchar);
var ucharPtrPtr = import_ref_napi.default.refType(ucharPtr);
var charPtr = import_ref_napi.default.refType(import_ref_napi.default.types.char);
var charPtrPtr = import_ref_napi.default.refType(charPtr);
var sizeT = import_ref_napi.default.types.size_t;
var sizeTPtr = import_ref_napi.default.refType(sizeT);
var lib = import_ffi_napi.default.Library("../../ffi/libkeylite_kv.so", {
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
var keylite_ffi_default = lib;

// keylite.ts
var import_ref_napi2 = __toESM(require_ref(), 1);
function checkResult(result, operation) {
  if (result !== 0 /* Ok */) {
    const errorMessages = {
      [1 /* ErrNull */]: "Null pointer error",
      [2 /* ErrIo */]: "I/O error",
      [3 /* ErrUtf8 */]: "UTF-8 encoding error",
      [4 /* ErrOther */]: "Unknown error"
    };
    throw new Error(`${operation} failed: ${errorMessages[result] || "Unknown error"}`);
  }
}

class KeyliteIterator {
  handle;
  db;
  constructor(handle, db) {
    this.handle = handle;
    this.db = db;
  }
  next() {
    const keyOut = import_ref_napi2.default.alloc(import_ref_napi2.default.refType(import_ref_napi2.default.types.uchar));
    const keyLenOut = import_ref_napi2.default.alloc(import_ref_napi2.default.types.size_t);
    const valOut = import_ref_napi2.default.alloc(import_ref_napi2.default.refType(import_ref_napi2.default.types.uchar));
    const valLenOut = import_ref_napi2.default.alloc(import_ref_napi2.default.types.size_t);
    const result = keylite_ffi_default.keylite_iter_next(this.handle, keyOut, keyLenOut, valOut, valLenOut);
    checkResult(result, "Iterator next");
    const keyPtr = keyOut.deref();
    const keyLen = keyLenOut.deref();
    if (import_ref_napi2.default.isNull(keyPtr) || keyLen === 0) {
      return null;
    }
    const valPtr = valOut.deref();
    const valLen = valLenOut.deref();
    const key = Buffer.from(import_ref_napi2.default.reinterpret(keyPtr, keyLen, 0));
    const value = Buffer.from(import_ref_napi2.default.reinterpret(valPtr, valLen, 0));
    keylite_ffi_default.keylite_free_value(keyPtr, keyLen);
    keylite_ffi_default.keylite_free_value(valPtr, valLen);
    return { key, value };
  }
  close() {
    if (this.handle) {
      keylite_ffi_default.keylite_iter_free(this.handle);
    }
  }
  *[Symbol.iterator]() {
    let item;
    while ((item = this.next()) !== null) {
      yield item;
    }
    this.close();
  }
}

class Keylite {
  handle = null;
  open(path) {
    if (this.handle) {
      throw new Error("Database is already open");
    }
    const dbOut = import_ref_napi2.default.alloc(import_ref_napi2.default.refType(import_ref_napi2.default.types.void));
    const result = keylite_ffi_default.keylite_open(path, dbOut);
    checkResult(result, "Open database");
    this.handle = dbOut.deref();
    if (import_ref_napi2.default.isNull(this.handle)) {
      throw new Error("Failed to open database");
    }
  }
  close() {
    if (!this.handle)
      return;
    keylite_ffi_default.keylite_close(this.handle);
    this.handle = null;
  }
  ensureOpen() {
    if (!this.handle) {
      throw new Error("Database is not open");
    }
  }
  put(key, value) {
    this.ensureOpen();
    const result = keylite_ffi_default.keylite_put(this.handle, key, key.length, value, value.length);
    checkResult(result, "Put");
  }
  get(key) {
    this.ensureOpen();
    const valOut = import_ref_napi2.default.alloc(import_ref_napi2.default.refType(import_ref_napi2.default.types.uchar));
    const valLenOut = import_ref_napi2.default.alloc(import_ref_napi2.default.types.size_t);
    const result = keylite_ffi_default.keylite_get(this.handle, key, key.length, valOut, valLenOut);
    checkResult(result, "Get");
    const valPtr = valOut.deref();
    const valLen = valLenOut.deref();
    if (import_ref_napi2.default.isNull(valPtr) || valLen === 0) {
      return null;
    }
    const value = Buffer.from(import_ref_napi2.default.reinterpret(valPtr, valLen, 0));
    keylite_ffi_default.keylite_free_value(valPtr, valLen);
    return value;
  }
  del(key) {
    this.ensureOpen();
    const result = keylite_ffi_default.keylite_del(this.handle, key, key.length);
    checkResult(result, "Delete");
  }
  putStr(key, value) {
    this.ensureOpen();
    const result = keylite_ffi_default.keylite_put_str(this.handle, key, value);
    checkResult(result, "Put string");
  }
  getStr(key) {
    this.ensureOpen();
    const valOut = import_ref_napi2.default.alloc(import_ref_napi2.default.refType(import_ref_napi2.default.types.char));
    const result = keylite_ffi_default.keylite_get_str(this.handle, key, valOut);
    checkResult(result, "Get string");
    const valPtr = valOut.deref();
    if (import_ref_napi2.default.isNull(valPtr)) {
      return null;
    }
    const value = import_ref_napi2.default.readCString(valPtr, 0);
    keylite_ffi_default.keylite_free_str(valPtr);
    return value;
  }
  delStr(key) {
    this.ensureOpen();
    const result = keylite_ffi_default.keylite_del_str(this.handle, key);
    checkResult(result, "Delete string");
  }
  scan(start, end) {
    this.ensureOpen();
    const iterOut = import_ref_napi2.default.alloc(import_ref_napi2.default.refType(import_ref_napi2.default.types.void));
    const startPtr = start ? start : null;
    const startLen = start ? start.length : 0;
    const endPtr = end ? end : null;
    const endLen = end ? end.length : 0;
    const result = keylite_ffi_default.keylite_scan(this.handle, startPtr, startLen, endPtr, endLen, iterOut);
    checkResult(result, "Scan");
    const iterHandle = iterOut.deref();
    return new KeyliteIterator(iterHandle, this);
  }
  scanStr(start, end) {
    this.ensureOpen();
    const iterOut = import_ref_napi2.default.alloc(import_ref_napi2.default.refType(import_ref_napi2.default.types.void));
    const result = keylite_ffi_default.keylite_scan_str(this.handle, start || null, end || null, iterOut);
    checkResult(result, "Scan string");
    const iterHandle = iterOut.deref();
    return new KeyliteIterator(iterHandle, this);
  }
}

// example.ts
var db = new Keylite;
db.open("testdb");
db.putStr("user:1", "tanish");
db.putStr("user:2", "vinayak");
db.putStr("user:3", "samyak");
db.putStr("user:4", "kanav");
db.putStr("user:5", "abhijeet");
var res = db.scanStr("user:2", "user:4");
for (const { key, value } of res) {
  console.log(key.toString(), value.toString());
}
