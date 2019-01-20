#include <fcntl.h>
#include <stdio.h>
#include <sys/stat.h>
#include <unistd.h>

int main(int argc, char ** argv) {
    for(int i = 0; i < argc; i++) {
        printf("Arg %d: %s\n", i, argv[i]);
    }

    printf("PID: %d\n", getpid());

    int fd = open("README.md", O_RDONLY);
    if (fd < 0) {
        perror("open");
        return 1;
    }

    printf("open %d\n", fd);

    struct stat statbuf;
    if(fstat(fd, &statbuf) < 0) {
        perror("fstat");
        return 1;
    }

    printf("stat dev: %d\n", statbuf.st_dev);
    printf("stat ino: %d\n", statbuf.st_ino);
    printf("stat mode: %d\n", statbuf.st_mode);
    printf("stat nlink: %d\n", statbuf.st_nlink);
    printf("stat uid: %d\n", statbuf.st_uid);
    printf("stat gid: %d\n", statbuf.st_gid);
    printf("stat size: %d\n", statbuf.st_size);
    printf("stat blksize: %d\n", statbuf.st_blksize);
    printf("stat blocks: %d\n", statbuf.st_blocks);
    printf("stat mtime: %d\n", statbuf.st_mtime);
    printf("stat atime: %d\n", statbuf.st_atime);
    printf("stat ctime: %d\n", statbuf.st_ctime);

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

    int rand = open("rand:", O_RDONLY);
    if (rand < 0) {
        perror("open rand");
        return 1;
    }

    printf("open rand %d\n", rand);

    char buf[1] = {0};
    if (read(rand, buf, 1) < 0) {
        perror("read rand");
        return 1;
    }

    printf("read rand: %d\n", (int)buf[0]);

    close(rand);

    return 0;
}
