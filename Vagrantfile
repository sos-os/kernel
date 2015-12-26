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
    curl -sf https://raw.githubusercontent.com/brson/multirust/master/blastoff.sh | sh -s -- --yes
    multirust default nightly-2015-11-19
  SHELL

  config.ssh.forward_x11 = true
end
