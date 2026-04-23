package main

import (
	"flag"
	"fmt"
	"os"
)

const version = "combat-solver prototype"

func main() {
	showVersion := flag.Bool("version", false, "print version and exit")
	flag.Parse()

	if *showVersion {
		fmt.Println(version)
		return
	}

	fmt.Fprintln(os.Stderr, "combat-solver: implementation not wired yet")
	os.Exit(2)
}
