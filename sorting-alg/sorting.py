import time
import random

def gen(n, r):
    list = []
    for i in range(0,n):
        list.append(random.randint(0,r))
    return list

def main(input):
    output = []
    temp = input[0]
    tempi = 0
    for i in range(len(input)-1):
        temp = input[0]
        tempi = 0
        for j in range(len(input)-1):
            if temp <= input[j]:
                temp = input[j]
                tempi = j
        output.append(input[tempi])
        del input[tempi]
    print(f'{output[:10]} length {len(output)}')

if __name__ == '__main__':
    starttime = time.time()
    main(gen(10000, 20000))
    endtime = time.time()
    print(f'took {(endtime-starttime)*1000} ms to run')