package main

import (
    "github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
    Use:   "app",
    Short: "My CLI app",
    Run: func(cmd *cobra.Command, args []string) {},
}

func main() {
    rootCmd.Execute()
}
