/*
 * Copyright 2021, Google LLC
 *
 * SPDX-License-Identifier: Apache-2.0
 */

// This file is a barebones, minimal-dependency test application that simply
// derefrences a null pointer to kill itself. It's primary use case is to test
// out KataOS' fault handlers.

#include <kata.h>

int main(int a0, int a1, int a2, int a3) {
  debug_printf("Goodbye, cruel world!\n");

  while (1) {
    char *p = 0x0;
    *p = 'g';
  }
}
