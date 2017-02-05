
    - path %path%;%USERPROFILE%\.cargo\bin
    - cargo rustc -- -Clink-args="/SUBSYSTEM:WINDOWS /ENTRY:mainCRTStartup"
