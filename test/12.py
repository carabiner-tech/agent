import math


def euler_problem_12():
    """Solves Project Euler Problem 12 by calculating and printing the value of the first triangle number to have over five hundred divisors."""

    def count_divisors(n):
        count = 0
        sqrt_n = int(math.sqrt(n))
        for i in range(1, sqrt_n + 1):
            if n % i == 0:
                count += 2  # i and n/i
        if sqrt_n * sqrt_n == n:
            count -= 1  # if n is a perfect square
        return count

    i = 1
    triangle_number = 0
    while True:
        triangle_number += i
        i += 1
        if count_divisors(triangle_number) > 500:
            break
    print(triangle_number)


if __name__ == "__main__":
    euler_problem_12()
