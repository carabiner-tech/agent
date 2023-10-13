from utils import is_prime

def euler_problem_3(number=600851475143):
    """Solves Project Euler Problem 3 by calculating and printing the largest prime factor of the specified number."""
    i = 2
    while i * i <= number:
        if not is_prime(i) or number % i != 0:
            i += 1
            continue
        number //= i
    print(number)

if __name__ == "__main__":
    euler_problem_3()