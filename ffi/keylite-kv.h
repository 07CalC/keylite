#pragma once

/* Generated with cbindgen:0.29.2 */

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string>
#define VERSION 1

typedef enum KeyliteResult {
  Ok = 0,
  ErrNull = 1,
  ErrIo = 2,
  ErrUtf8 = 3,
  ErrOther = 4,
} KeyliteResult;

typedef struct KeyliteDb KeyliteDb;

typedef struct KeyliteIterator KeyliteIterator;

void keylite_close(struct KeyliteDb *Db);

enum KeyliteResult keylite_del(struct KeyliteDb *Db, const uint8_t *Key,
                               size_t KeyLen);

enum KeyliteResult keylite_del_str(struct KeyliteDb *Db, const char *Key);

void keylite_free_str(char *Val);

void keylite_free_value(uint8_t *Val, size_t Len);

enum KeyliteResult keylite_get(struct KeyliteDb *Db, const uint8_t *Key,
                               size_t KeyLen, uint8_t **ValOut,
                               size_t *ValLenOut);

enum KeyliteResult keylite_get_str(struct KeyliteDb *Db, const char *Key,
                                   char **ValOut);

void keylite_iter_free(struct KeyliteIterator *Iter);

enum KeyliteResult keylite_iter_next(struct KeyliteIterator *Iter,
                                     uint8_t **KeyOut, size_t *KeyLenOut,
                                     uint8_t **ValOut, size_t *ValLenOut);

enum KeyliteResult keylite_open(const char *Path, struct KeyliteDb **DbOut);

enum KeyliteResult keylite_put(struct KeyliteDb *Db, const uint8_t *Key,
                               size_t KeyLen, const uint8_t *Val,
                               size_t ValLen);

enum KeyliteResult keylite_put_str(struct KeyliteDb *Db, const char *Key,
                                   const char *Val);

enum KeyliteResult keylite_scan(struct KeyliteDb *Db, const uint8_t *Start,
                                size_t StartLen, const uint8_t *End,
                                size_t EndLen,
                                struct KeyliteIterator **IterOut);

enum KeyliteResult keylite_scan_str(struct KeyliteDb *Db, const char *Start,
                                    const char *End,
                                    struct KeyliteIterator **IterOut);
