# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure(2) do |config|
  config.vm.box = "hashicorp/precise64"

  config.vm.provision "shell", inline: <<-SHELL
    sudo apt-get update
    sudo apt-get install -y build-essential
    sudo apt-get install -y curl
    sudo apt-get install nasm -y
    sudo apt-get install xorriso -y
    sudo apt-get install git -y
    sudo apt-get install vim -y
    sudo apt-get install -y qemu
    su - vagrant -c 'curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly'
    ex +'$s@$@\rexport PATH=~/.cargo/bin/:$PATH@' -cwq /etc/bash.bashrc
  SHELL

  config.ssh.forward_x11 = true
end
