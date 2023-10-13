def euler_problem_1(limit=1000):
    """Solves Project Euler Problem 1 by calculating and printing the sum of all the multiples of 3 or 5 below the specified limit."""
    total = 0
    for x in range(limit):
        if x % 3 == 0 or x % 5 == 0:
            total += x
    print(total)

if __name__ == "__main__":
    euler_problem_1()