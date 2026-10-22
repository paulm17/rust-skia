// Supplemental contract checks, separate from original registrations.
#include "include/core/SkData.h"
#include "include/core/SkStream.h"
#include "src/core/SkArenaAlloc.h"
#include "src/core/SkChecksum.h"
#include "tests/FakeStreams.h"
#include <cstdio>
#include <cstring>
int m02_contracts(){
 int fails=0;
 auto check=[&](const char*name,bool ok){std::printf("CONTRACT %s %s\n",name,ok?"PASS":"FAIL");fails+=!ok;};
 for(const char* bytes:{"","\xfe","\xfe\x01","\xff\x01\x02\x03"}){SkMemoryStream s(bytes,strlen(bytes),true);size_t value=999;check("packed_truncation",!s.readPackedUInt(&value));}
 unsigned char input[]={1,2,3,4};auto d=SkData::MakeWithoutCopy(input,4);check("without_copy_identity",d->data()==input);
 int releases=0;{auto d=SkData::MakeWithProc(nullptr,0,[](const void*,void* c){++*static_cast<int*>(c);},&releases);}check("empty_data_release",releases==1);
 alignas(16) char storage[1000];SkArenaAlloc a(storage,sizeof(storage),1000);auto p=a.make<int>(3);check("arena_external_storage",reinterpret_cast<char*>(p)>=storage&&reinterpret_cast<char*>(p)<storage+sizeof(storage));
 SkDynamicMemoryWStream w;w.write16(0x1234);w.write32(0x89abcdef);auto data=w.detachAsData();unsigned char expected[]={0x34,0x12,0xef,0xcd,0xab,0x89};check("byte_order",data->size()==6&&!memcmp(data->data(),expected,6));
 for(const char*text:{"","a","abc","abcdefghijklmnopqrstuvwxyz"})for(unsigned seed:{0u,1u,0xffffffffu}){std::printf("CHECKSUM %s %u %u\n",*text?text:"<empty>",seed,SkChecksum::Hash32(text,strlen(text),seed));}
 return fails;
}
