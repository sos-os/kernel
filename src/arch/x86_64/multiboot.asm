; SOS multiboot header
;
; based on code by Phil Oppermann
; (http://blog.phil-opp.com/rust-os/multiboot-kernel.html)
;
; Licensed under the Apache License, Version 2.0 (the "License");
; you may not use this file except in compliance with the License.
; You may obtain a copy of the License at
;
;    http://www.apache.org/licenses/LICENSE-2.0
;
; Unless required by applicable law or agreed to in writing, software
; distributed under the License is distributed on an "AS IS" BASIS,
; WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
; See the License for the specific language governing permissions and
; limitations under the License.

%define MULTIBOOT_2_MAGIC   0xe85250d6
%define HEADER_LEN          header_end - header_begin

section .multiboot_header
header_begin:
    dd MULTIBOOT_2_MAGIC    ; multiboot 2 magic
    dd 0                    ; arch 0 (i386)
    dd HEADER_LEN           ; header length

    ; checksum
    dd -(MULTIBOOT_2_MAGIC + 0 + HEADER_LEN)

    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 8    ; size
header_end:
