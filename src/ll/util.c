// TODO: big endian archtecture support
int be(int x){
   return  ((x & 0xff) << 24) | (((x >> 8) & 0xff) << 16) |
           (((x >> 16) & 0xff) << 8) | ((x >> 24) & 0xff);
}

int be16(int x){
   return  ((x & 0xff) << 8) | ((x >> 8) & 0xff);
}

int ldw(unsigned char* x, int offset){
    int v = *(int *)(x+offset);
    return be(v);
}

int ldh(unsigned char* x, int offset){
    int v = ((*(x+offset)) << 8) | (*(x+offset+1));
    return be16(v);
}

int ldb(unsigned char* x, int offset){
    return *(x+offset);
}

int msh(unsigned char* x, int offset){
    return (ldb(x, offset) & 0xf) << 2;
}
