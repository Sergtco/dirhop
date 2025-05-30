# DirHop

ls + cd in one command.  
Use two-character input like in [amp](https://amp.rs/)

## Example

Usage

```console
~/ $ cd "$(dirhop)"
[aa]Documents
[ab]Pictures
[ac]Programming
[ad]Downloads
>ab<CR>

~/Pictures $
```

## Supported commands

`CTRL+H` - Show hidden.  
`CTRL+C` - Quit.  
`ESC` - Clear.  
`..` - Go to parent directory.  
`>>` - Next page.  
`<<` - Prev page.
