from typing import List, Union
from math import gcd


def is_prime(n: int) -> bool:
    if n < 2:
        return False
    for i in range(2, int(n ** 0.5) + 1):
        if n % i == 0:
            return False
    return True


def generate_primes(limit: int) -> List[int]:
    sieve = [True] * (limit + 1)
    sieve[0] = sieve[1] = False
    for num in range(2, int(limit ** 0.5) + 1):
        if sieve[num]:
            for multiple in range(num * num, limit + 1, num):
                sieve[multiple] = False
    return [num for num, is_prime in enumerate(sieve) if is_prime]


def lcm(*args: int) -> int:
    result = 1
    for num in args:
        result = result * num // gcd(result, num)
    return result


def is_palindrome(s: Union[int, str]) -> bool:
    s = str(s)
    return s == s[::-1]


def generate_fibonacci(limit: int) -> List[int]:
    fibs = [1, 2]
    while fibs[-1] + fibs[-2] < limit:
        fibs.append(fibs[-1] + fibs[-2])
    return fibs