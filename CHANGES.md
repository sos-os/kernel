<a name="elf-v0-0-2"></a>
## elf v0.0.2 (2017-03-15)


#### Bug Fixes

* **elf:**
  *  add assertion that extract_from_slice offset is T-aligned ([e7df6620](e7df6620))
  *  fix arbitrary lifetime in elf::extract_from_slice ([60ed6a15](60ed6a15))
  *  fix a typo in elf::file::Class ([99332049](99332049))
  *  fix validation of elf section word size ([15eb3664](15eb3664))
  *  make section header name offset a Word ([1583b718](1583b718))
  *  fix a typo in elf::file::Class ([cbb9ca9d](cbb9ca9d))
  *  fix validation of elf section word size ([cec0a13b](cec0a13b))
  *  make section header name offset a Word ([89c4edd3](89c4edd3))
* **x86_64:**  make kernel_init use re-written section headers ([19f75fe9](19f75fe9))

#### Features

* **elf:**
  *  rewrite SectionHeader to be trait-based ([9d2eb03c](9d2eb03c), closes [#93](93))
  *  add default type parameters for Image, default word type ([4c09e7ad](4c09e7ad))
  *  changed extract_from_slice() to return a Result ([5e359b01](5e359b01))
  *  start on convert::TryFrom<&'a [u8]> for elf image ([34fba817](34fba817))
  *  add program headers slice ref to elf image ([caaf3245](caaf3245))
  *  add struct representing 32-bit program header ([cc3f38af](cc3f38af))
  *  add struct representation of 64-bit program header ([ee718532](ee718532))
  *  begin implementing elf program header ([00d49e41](00d49e41))
  *  add getters for more file header fields ([888d99fa](888d99fa))
  *  add getters for returning file header fields as usize ([e2cf28dc](e2cf28dc))
  *  add function to get the section header string table ([48036533](48036533))
  *  ELF String Table made indexable ([c03e36f0](c03e36f0))
  *  first pass on parsing ELF string tables ([5c266f0f](5c266f0f), closes [#83](83))
  *  nicer handling of ELF sections with invalid type fields ([9083e9ba](9083e9ba))
  *  add fmt::Display implementation for ELF sections ([4eb34a3a](4eb34a3a))
  *  add getters for more file header fields ([ae29fe71](ae29fe71))
  *  add getters for returning file header fields as usize ([3b6078fd](3b6078fd))
  *  add function to get the section header string table ([a8002040](a8002040))
  *  ELF String Table made indexable ([966d683b](966d683b))
  *  first pass on parsing ELF string tables ([9e5b195d](9e5b195d), closes [#83](83))
  *  nicer handling of ELF sections with invalid type fields ([8c07e373](8c07e373))
  *  add fmt::Display implementation for ELF sections ([50c04cea](50c04cea))
* **multiboot2:**  add IntoIter implementations for tags ([4a546e7a](4a546e7a))
* **x86_all:**
  *  multiboot2 mem areas iterator doesn't skip ACPI areas ([3310f1fc](3310f1fc))
  *  multiboot2 mem areas iterator doesn't skip ACPI areas ([2487f494](2487f494))
