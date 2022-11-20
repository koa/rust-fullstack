FROM scratch
COPY target/x86_64-unknown-linux-gnu/release/backend-artifact /

# have to use exec form as we have no shell to execute to execute our binary
CMD ["/backend-artifact"]