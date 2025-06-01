let
  greeting = "Nix";
  greet = name: "Hello, ${name}!";
in
greet greeting
