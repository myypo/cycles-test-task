############
### TAGS ###
############

# VALID: Create a new tag
POST {{url}}/api/v1/tags
{
  "name": "Meaty"
}
HTTP 201
[Captures]
tag_id1: jsonpath "$.data.id"

# ERROR: Try to create a tag with the same name
POST {{url}}/api/v1/tags
{
  "name": "meaTy"
}
HTTP 409

# VALID: Create a new tag
POST {{url}}/api/v1/tags
{
  "name": "Testy"
}
HTTP 201
[Captures]
tag_id2: jsonpath "$.data.id"

# ERROR: Try to get an inexistent tag
GET {{url}}/api/v1/tags/abefd7d1-8c94-41a3-ab74-ae1869813622
HTTP 404

# VALID: Get all tags
GET {{url}}/api/v1/tags
HTTP 200
[Asserts]
jsonpath "$.data.count" == 2
jsonpath "$.data.list" exists

###################
### INGREDIENTS ###
###################

# VALID: Create a new ingredient
POST {{url}}/api/v1/ingredients
{
  "name": "Tuna"
}
HTTP 201
[Captures]
ingredient_id1: jsonpath "$.data.id"

# ERROR: Try to create an ingredient with the same name
POST {{url}}/api/v1/ingredients
{
  "name": "tuna"
}
HTTP 409

# VALID: Add an image to the created ingredient
POST {{url}}/api/v1/ingredients/{{ingredient_id1}}/images
[MultipartFormData]
image: file,default.hurl;
HTTP 201
[Asserts]
jsonpath "$.data.name" == "Tuna"
jsonpath "$.data.image_list" count == 1

# ERROR: Do not attach an image in multiform
POST {{url}}/api/v1/ingredients/{{ingredient_id1}}/images
HTTP 400

# VALID: Create a new ingredient
POST {{url}}/api/v1/ingredients
{
  "name": "Garlic"
}
HTTP 201
[Captures]
ingredient_id2: jsonpath "$.data.id"

# VALID: Get created ingredient by ID
GET {{url}}/api/v1/ingredients/{{ingredient_id1}}
HTTP 200
[Asserts]
jsonpath "$.data.name" == "Tuna"
jsonpath "$.data.image_list" count == 1
jsonpath "$.data.id" == {{ingredient_id1}}

# ERROR: Try to get an inexistent ingredient
GET {{url}}/api/v1/ingredients/abefd7d1-8c94-41a3-ab74-ae1869813622
HTTP 404

# VALID: Get all ingredients
GET {{url}}/api/v1/ingredients
HTTP 200
[Asserts]
jsonpath "$.data.count" == 2
jsonpath "$.data.list" exists

###############
### BURGERS ###
###############

# VALID: Create a new burger
POST {{url}}/api/v1/burgers
{
  "name": "Big Testy",
  "tag_id_list": ["{{tag_id1}}", "{{tag_id2}}"],
  "ingredient_id_list": ["{{ingredient_id1}}", "{{ingredient_id2}}"]
}
HTTP 201
[Asserts]
jsonpath "$.data.name" == "Big Testy"
jsonpath "$.data.tag_list" count == 2
jsonpath "$.data.ingredient_list" count == 2
[Captures]
burger_id1: jsonpath "$.data.id"

# VALID: Add an image to the created burger
POST {{url}}/api/v1/burgers/{{burger_id1}}/images
[MultipartFormData]
image: file,default.hurl;
HTTP 201
[Asserts]
jsonpath "$.data.name" == "Big Testy"
jsonpath "$.data.tag_list" count == 2
jsonpath "$.data.ingredient_list" count == 2
jsonpath "$.data.image_list" count == 1

# ERROR: Do not attach an image in multiform
POST {{url}}/api/v1/burgers/{{burger_id1}}/images
HTTP 400

# VALID: Add another image to the created burger
POST {{url}}/api/v1/burgers/{{burger_id1}}/images
[MultipartFormData]
image: file,default.hurl;
HTTP 201
[Asserts]
jsonpath "$.data.name" == "Big Testy"
jsonpath "$.data.tag_list" count == 2
jsonpath "$.data.ingredient_list" count == 2
jsonpath "$.data.image_list" count == 2

# VALID: Get created burger by ID
GET {{url}}/api/v1/burgers/{{burger_id1}}
HTTP 200
[Asserts]
jsonpath "$.data.name" == "Big Testy"
jsonpath "$.data.tag_list" count == 2
jsonpath "$.data.ingredient_list" count == 2
jsonpath "$.data.image_list" count == 2
jsonpath "$.data.id" == {{burger_id1}}

# ERROR: Try to get an inexistent burger
GET {{url}}/api/v1/burgers/abefd7d1-8c94-41a3-ab74-ae1869813622
HTTP 404

# VALID: Create a new burger
POST {{url}}/api/v1/burgers
{
  "name": "Chu",
  "tag_id_list": ["{{tag_id1}}"],
  "ingredient_id_list": ["{{ingredient_id1}}"]
}
HTTP 201
[Asserts]
jsonpath "$.data.name" == "Chu"
jsonpath "$.data.tag_list" count == 1
jsonpath "$.data.ingredient_list" count == 1
[Captures]
burger_id2: jsonpath "$.data.id"

# VALID: Get all burgers
GET {{url}}/api/v1/burgers
HTTP 200
[Asserts]
jsonpath "$.data.count" == 2
jsonpath "$.data.list" exists

# VALID: Get burger list starting with C
GET {{url}}/api/v1/burgers?first_letter_name=c
HTTP 200
[Asserts]
jsonpath "$.data.count" == 1
jsonpath "$.data.list" exists
jsonpath "$.data.list[0].tag_list" count == 1
jsonpath "$.data.list[0].ingredient_list" count == 1

# VALID: Get burger list containing Garlic
GET {{url}}/api/v1/burgers?ingredient_name_list=garlic
HTTP 200
[Asserts]
jsonpath "$.data.count" == 1
jsonpath "$.data.list" exists
jsonpath "$.data.list[0].tag_list" count == 2
jsonpath "$.data.list[0].ingredient_list" count == 2

# VALID: Get burger list tagged as Testy
GET {{url}}/api/v1/burgers?tag_name_list=testy
HTTP 200
[Asserts]
jsonpath "$.data.count" == 1
jsonpath "$.data.list" exists
jsonpath "$.data.list[0].tag_list" count == 2
jsonpath "$.data.list[0].ingredient_list" count == 2

# VALID: Get all random burgers
GET {{url}}/api/v1/burgers/random
HTTP 200
[Asserts]
jsonpath "$.data.count" == 2
jsonpath "$.data.list" exists
jsonpath "$.data.list[0].tag_list" count <= 2
jsonpath "$.data.list[0].ingredient_list" count <= 2
jsonpath "$.data.list[1].tag_list" count <= 2
jsonpath "$.data.list[1].ingredient_list" count <= 2

# VALID: Get 1 random burger
GET {{url}}/api/v1/burgers/random?limit=1
HTTP 200
[Asserts]
jsonpath "$.data.count" == 1
jsonpath "$.data.list" exists
jsonpath "$.data.list[0].tag_list" count <= 2
jsonpath "$.data.list[0].ingredient_list" count <= 2
