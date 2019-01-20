#include <fcntl.h>
#include <stdio.h>
#include <sys/stat.h>
#include <unistd.h>

int main(int argc, char ** argv) {
    for(int i = 0; i < argc; i++) {
        printf("%d: %s\n", i, argv[i]);
    }

    int fd = open("README.md", O_RDONLY);
    if (fd < 0) {
        perror("open");
        return 1;
    }

    printf("open %d\n", fd);

    struct stat buf;
    if(fstat(fd, &buf) < 0) {
        perror("fstat");
        return 1;
    }

    printf("dev: %d\n", buf.st_dev);
    printf("ino: %d\n", buf.st_ino);
    printf("mode: %d\n", buf.st_mode);
    printf("nlink: %d\n", buf.st_nlink);
    printf("uid: %d\n", buf.st_uid);
    printf("gid: %d\n", buf.st_gid);
    printf("size: %d\n", buf.st_size);
    printf("blksize: %d\n", buf.st_blksize);
    printf("blocks: %d\n", buf.st_blocks);
    printf("mtime: %d\n", buf.st_mtime);
    printf("atime: %d\n", buf.st_atime);
    printf("ctime: %d\n", buf.st_ctime);

    while (1) {
        char buf[256] = { 0 };
        int count = read(fd, buf, 256);
        if (count < 0) {
            perror("read");
            return 1;
        } else if (count == 0) {
            break;
        }

        write(1, buf, count);
    }

    close(fd);

    return 0;
}
