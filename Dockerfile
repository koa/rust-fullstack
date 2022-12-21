FROM scratch
COPY target/release/backend-artifact /

# have to use exec form as we have no shell to execute to execute our binary
CMD ["/backend-artifact"]