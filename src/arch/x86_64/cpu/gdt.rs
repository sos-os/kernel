use arch::cpu::segment;

const GDT_SIZE: usize = 512;

type Gdt = [segment::Descriptor; GDT_SIZE];
