import math
from utils import generate_primes

def euler_problem_7(position=10001):
    """Solves Project Euler Problem 7 by calculating and printing the 10,001st prime number."""
    upper_bound = int(position * (math.log(position) + math.log(math.log(position))))  # More accurate estimate of the upper bound for the nth prime
    primes = generate_primes(upper_bound)
    print(primes[position - 1])


if __name__ == "__main__":
    euler_problem_7()