def euler_problem_8():
    """Solves Project Euler Problem 8 by finding and printing the thirteen adjacent digits in the 1000-digit number that have the greatest product."""
    number = '731671765313306249192251196744265747423553491949349698352031277450632623957831801698480186947885184385861560789112949495459501737958331952853208805511254748258954930281907213852623466744345357326721478444856909275425675400442572261443812807977156914359977001242890987654321'
    greatest_product = 0
    for i in range(len(number) - 12):
        product = 1
        for j in range(13):
            product *= int(number[i + j])
        greatest_product = max(greatest_product, product)
    print(greatest_product)

if __name__ == "__main__":
    euler_problem_8()