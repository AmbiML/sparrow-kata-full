#
# Copyright 2018, Data61
# Commonwealth Scientific and Industrial Research Organisation (CSIRO)
# ABN 41 687 119 230.
#
# This software may be distributed and modified according to the terms of
# the BSD 2-Clause license. Note that NO WARRANTY is provided.
# See "LICENSE_BSD2.txt" for details.
#
# @TAG(DATA61_BSD)
#

cmake_minimum_required(VERSION 3.7.2)

project(system)

CAmkESAddImportPath(interfaces)

includeGlobalComponents()
include(${CMAKE_CURRENT_LIST_DIR}/rust.cmake)

# Note: the includes generated for each CAmkES component listed here are
# generated separately from the CMake build scripts. See also build/kata.mk for
# details, specifically the kata-component-headers target.

RustAddLibrary(
  kata_debug_console
  SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/components/DebugConsole
  LIB_FILENAME libkata_debug_console.a
)

DeclareCAmkESComponent(DebugConsole
  LIBS kata_debug_console
  INCLUDES interfaces
  $ENV{OUT}/kata/components
)

RustAddLibrary(
  kata_memory_manager
  SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/components/MemoryManager
  LIB_FILENAME libkata_memory_manager.a
)

DeclareCAmkESComponent(MemoryManager
  LIBS kata_memory_manager
  INCLUDES interfaces
  $ENV{OUT}/kata/components
)

RustAddLibrary(
  kata_ml_coordinator
  SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/components/MlCoordinator
  LIB_FILENAME libkata_ml_component.a
)

DeclareCAmkESComponent(MlCoordinator
  LIBS kata_ml_coordinator
  INCLUDES interfaces
  $ENV{OUT}/kata/components
)

RustAddLibrary(
  kata_process_manager
  SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/components/ProcessManager
  LIB_FILENAME libkata_process_manager.a
)

DeclareCAmkESComponent(ProcessManager
  LIBS kata_process_manager
  INCLUDES interfaces
  $ENV{OUT}/kata/components
)

RustAddLibrary(
  kata_security_coordinator
  SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/components/SecurityCoordinator
  LIB_FILENAME libkata_security_coordinator.a
)

DeclareCAmkESComponent(SecurityCoordinator
  LIBS kata_security_coordinator
  INCLUDES interfaces
  $ENV{OUT}/kata/components
)

RustAddLibrary(
  kata_storage_manager
  SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/components/StorageManager
  LIB_FILENAME libkata_storage_manager.a
)

DeclareCAmkESComponent(StorageManager
  LIBS kata_storage_manager
  INCLUDES interfaces
  $ENV{OUT}/kata/components
)

RustAddLibrary(
  kata_timer_service
  SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/components/TimerService
  LIB_FILENAME libkata_timer_component.a
)

DeclareCAmkESComponent(TimerService
  LIBS kata_timer_service
  INCLUDES interfaces
  $ENV{OUT}/kata/components
)

RustAddLibrary(
  mailbox_driver
  SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/components/MailboxDriver
  LIB_FILENAME libmailbox_driver.a
)

DeclareCAmkESComponent(MailboxDriver
  LIBS mailbox_driver
  INCLUDES interfaces
  $ENV{OUT}/kata/components
)

DeclareCAmkESComponent(LogFibonacci
  SOURCES
  components/LogFibonacci/src/main.c
)

DeclareCAmkESComponent(OpenTitanUARTDriver
  SOURCES
  components/OpenTitanUARTDriver/src/driver.c
  components/OpenTitanUARTDriver/src/circular_buffer.c
  INCLUDES
  opentitan-gen/include
  components/OpenTitanUARTDriver/include
  $ENV{OUT}/kata/components
)

DeclareCAmkESComponent(VectorCoreDriver
  SOURCES
  components/VectorCoreDriver/src/driver.c
  INCLUDES
  vc_top-gen/include
  $ENV{OUT}/kata/components
)

DeclareCAmkESRootserver(system.camkes)
