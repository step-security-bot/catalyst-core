# Amazon AWS EKS and RDS PostgreSQL with terraform

See:

* <https://dzone.com/articles/amazon-aws-eks-and-rds-postgresql-with-terraform-i>
* <https://aws.amazon.com/blogs/database/deploy-amazon-rds-databases-for-applications-in-kubernetes/>
* <https://github.com/mudrii/eks_rds_terraform>
* <https://learnk8s.io/terraform-eks>
* <https://github.com/k-mitevski/terraform-k8s>

Assuming you already have Amazon AWS account we will need additional binaries
for AWS CLI, terraform, kubectl and aws-iam-authenticator.

This aws kubernetes installation is not currently nixified. It is a complete
kubernetes deployment as a starting point for iteration and automation.  It is
customized to a Catalyst use case.

Todo:

* [ ] Use external RDS, and stop creating an RDS instance.
* [ ] Deploy Catalyst Containers.
* [ ] Nixify
* [ ] Automate

**Article is structured in 5 parts**

* Initial tooling setup aws cli , kubectl and terraform
* Creating terraform IAM account with access keys and access policy
* Creating back-end storage for tfstate file in AWS S3
* Creating Kubernetes cluster on AWS EKS and RDS on PostgreSQL
* Working with kubernetes "kubectl" in EKS

## Initial tooling setup aws-cli, kubectl, terraform and aws-iam-authenticator

Assuming you already have AWS account and [AWS CLI
installed](https://docs.aws.amazon.com/cli/latest/userguide/awscli-install-linux.html)
and [AWS CLI
configured](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-getting-started.html)
for your user account we will need additional binaries for, terraform and
kubectl.

### Deploying terraform

#### terraform for OS X

```sh
curl -o terraform_0.11.7_darwin_amd64.zip \
https://releases.hashicorp.com/terraform/0.11.7/terraform_0.11.7_darwin_amd64.zip

unzip terraform_0.11.7_linux_amd64.zip -d /usr/local/bin/
```

#### terraform for Linux

```sh
curl https://releases.hashicorp.com/terraform/0.11.7/terraform_0.11.7_linux_amd64.zip > terraform_0.11.7_linux_amd64.zip

unzip terraform_0.11.7_linux_amd64.zip -d /usr/local/bin/
```

#### terraform installation verification

Verify terraform version 0.11.7 or higher is installed:

```sh
terraform version
```

### Deploying kubectl

#### kubectl for OS X

```sh
curl -o kubectl https://storage.googleapis.com/kubernetes-release/release/v1.11.0/bin/darwin/amd64/kubectl

chmod +x kubectl

sudo mv kubectl /usr/local/bin/
```

#### kubectl for Linux

```sh
wget https://storage.googleapis.com/kubernetes-release/release/v1.11.0/bin/linux/amd64/kubectl

chmod +x kubectl

sudo mv kubectl /usr/local/bin/
```

#### kubectl installation verification

```sh
kubectl version --client
```

### Deploying aws-iam-authenticator

[aws-iam-authenticator](https://github.com/kubernetes-sigs/aws-iam-authenticator)
is a tool developed by [Heptio](https://heptio.com/) Team and this tool will
allow us to manage EKS by using kubectl

#### aws-iam-authenticator for OS X

```sh
curl -o aws-iam-authenticator \
https://amazon-eks.s3-us-west-2.amazonaws.com/1.10.3/2018-07-26/bin/darwin/amd64/aws-iam-authenticator

chmod +x ./aws-iam-authenticator

cp ./aws-iam-authenticator $HOME/bin/aws-iam-authenticator && export PATH=$HOME/bin:$PATH
```

#### aws-iam-authenticator for Linux

```sh
curl -o aws-iam-authenticator \
https://amazon-eks.s3-us-west-2.amazonaws.com/1.10.3/2018-07-26/bin/linux/amd64/aws-iam-authenticator

chmod +x ./aws-iam-authenticator

cp ./aws-iam-authenticator $HOME/.local/bin/aws-iam-authenticator && export PATH=$HOME/bin:$PATH
```

#### aws-iam-authenticator installation verification

```sh
aws-iam-authenticator help
```

### Authenticate to AWS

```sh
aws configure
```

## Creating terraform IAM account with access keys and access policy

1st step is to setup terraform admin account in AWS IAM

### Create IAM terraform User

```sh
aws iam create-user --user-name terraform
```

### Add to newly created terraform user IAM admin policy

> NOTE: For production or event proper testing account you may need tighten up
> and restrict access for terraform IAM user

```sh
aws iam attach-user-policy --user-name terraform --policy-arn arn:aws:iam::aws:policy/AdministratorAccess
```

### Create access keys for the user

> NOTE: This Access Key and Secret Access Key will be used by terraform to
> manage infrastructure deployment

```sh
aws iam create-access-key --user-name terraform
```

### update terraform.tfvars file with access and security keys

Create access and security keys for newly created terraform IAM account:

#### Example Command Sequence

```bash
$ aws iam create-user --user-name terraform
{
  "User": {
    "UserName":"terraform",
    "Path": "/",
    "CreateDate":"2018-08-10T05:37:17Z",
    "UserId": "AIDAIH2ODKZ56KSLZANH2",
    "Arn": "arn:aws:iam::092405883625:user/terraform"
  }
}

$ aws iam attach-user-policy --user-name terraform --policy-arn arn:aws:iam::aws:policy/AdministratorAccess

$ aws iam create-access-key --user-name terraform

{
    "AccessKey":
    {
        "UserName": "terraform",
        "Status": "Active",
        "CreateDate": "2018-08-10T05:37:35Z",
        "SecretAccessKey": "+fjHF3v9/XwXESs0EB7gEJh9WJBMAaRTxxw+/EGV","AccessKeyId":"AKIAJPV22B7WDFYOXEOQ"
    }
}

$ echo 'access_key  = "AKIAJPV22B7WDFYOXEOQ"' >> terraform.tfvars
$ echo 'secret_key  = "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"' >> terraform.tfvars
$ cat terraform.tfvars
access_key  = "AKIAJPV22B7WDFYOXEOQ"
secret_key  = "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"
```

## Creating back-end storage for tfstate file in AWS S3

Once we have terraform IAM account created we can proceed to next step creating
dedicated bucket to keep terraform state files

### Create terraform state bucket

> NOTE: Change name of the bucker, name should be unique across all AWS S3 buckets

```sh
aws s3 mb s3://catalyst-dev-tfstate --region eu-west-2
```

### Enable versioning on the newly created bucket

```sh
aws s3api put-bucket-versioning --bucket catalyst-dev-tfstate --versioning-configuration Status=Enabled
```

## Creating Kubernetes cluster on AWS EKS and RDS on PostgreSQL

Now we can move into creating new infrastructure, eks and rds with terraform

```sh
    .
    ├── backend.tf
    ├── eks
    │   ├── eks_cluster
    │   │   ├── main.tf
    │   │   ├── outputs.tf
    │   │   └── variables.tf
    │   ├── eks_iam_roles
    │   │   ├── main.tf
    │   │   └── outputs.tf
    │   ├── eks_node
    │   │   ├── main.tf
    │   │   ├── outputs.tf
    │   │   ├── userdata.tpl
    │   │   └── variables.tf
    │   └── eks_sec_group
    │       ├── main.tf
    │       ├── outputs.tf
    │       └── variables.tf
    ├── main.tf
    ├── network
    │   ├── route
    │   │   ├── main.tf
    │   │   ├── outputs.tf
    │   │   └── variables.tf
    │   ├── sec_group
    │   │   ├── main.tf
    │   │   ├── outputs.tf
    │   │   └── variables.tf
    │   ├── subnets
    │   │   ├── main.tf
    │   │   ├── outputs.tf
    │   │   └── variables.tf
    │   └── vpc
    │       ├── main.tf
    │       ├── outputs.tf
    │       └── variables.tf
    ├── outputs.tf
    ├── rds
    │   ├── main.tf
    │   ├── outputs.tf
    │   └── variables.tf
    ├── README.md
    ├── terraform.tfvars
    ├── variables.tf
    └── yaml
        ├── eks-admin-cluster-role-binding.yaml
        └── eks-admin-service-account.yaml
```

We will use terraform modules to keep our code clean and organized Terraform
will run 2 separate environment dev and prod using same sources only difference
in this case is number of worker nodes for kubernetes.

See: [variables.tf](./variables.tf)

Terraform modules will create

* VPC
* Subnets
* Routes
* IAM Roles for master and nodes
* Security Groups "Firewall" to allow master and nodes to communicate
* EKS cluster
* Autoscaling Group will create nodes to be added to the cluster
* Security group for RDS
* RDS with PostgreSQL

> NOTE: very important to keep tags as if tags is not specify nodes will not be
> able to join cluster

### Initial setup create and create new workspace for terraform

cd into project folder and create workspace for dev and prod

#### Initialize and pull terraform cloud specific dependencies

```sh
$ terraform init
Initializing modules...

Initializing the backend...

Initializing provider plugins...
- Finding latest version of hashicorp/aws...
- Finding latest version of hashicorp/null...
- Finding latest version of hashicorp/template...
- Installing hashicorp/aws v4.34.0...
- Installed hashicorp/aws v4.34.0 (signed by HashiCorp)
- Installing hashicorp/null v3.1.1...
- Installed hashicorp/null v3.1.1 (signed by HashiCorp)
- Installing hashicorp/template v2.2.0...
- Installed hashicorp/template v2.2.0 (signed by HashiCorp)

Terraform has created a lock file .terraform.lock.hcl to record the provider
selections it made above. Include this file in your version control repository
so that Terraform can guarantee to make the same selections by default when
you run "terraform init" in the future.

Terraform has been successfully initialized!

You may now begin working with Terraform. Try running "terraform plan" to see
any changes that are required for your infrastructure. All Terraform commands
should now work.

If you ever set or change modules or backend configuration for Terraform,
rerun this command to reinitialize your working directory. If you forget, other
commands will detect it and remind you to do so if necessary.

```

> Note: IF you get an `Error refreshing state: AccessDenied: Access Denied` make
> sure you are using the correct aws profile.  This will occur if your default
> profile does not have enough permissions.  To change AWS profile precede
> terraform calls with `AWS_PROFILE=<profile>`\
> eg: `$ AWS_PROFILE=admin terraform init`

#### Create dev workspace

```sh
$ terraform workspace new dev
Created and switched to workspace "dev"!

You're now on a new, empty workspace. Workspaces isolate their state,
so if you run "terraform plan" Terraform will not see any existing state
for this configuration.
```

#### List available workspace

```sh
$ terraform workspace list
  default
* dev
```

#### Select dev workspace

```sh
terraform workspace select dev
```

Before we can start will need to update variables and add db password to terraform.tfvars

```sh
echo 'db_password = "Your_DB_Passwd."' >> terraform.tfvars
```

#### It's a good idea to sync terraform modules

```sh
terraform get -update
```

### View terraform plan

```sh
terraform plan
```

### Apply terraform plan

> NOTE: building complete infrastructure may take more than 10 minutes.

```sh
terraform apply
```

[![asciicast](https://asciinema.org/a/195802.png)](https://asciinema.org/a/195802)

### Verify instance creation

```sh
aws ec2 describe-instances --output table
```

### We are not done yet

#### Create new AWS CLI profile

In order to use kubectl with EKS we need to set new AWS CLI profile

> NOTE: will need to use secret and access keys from terraform.tfvars

```sh
cat terraform.tfvars

aws configure --profile terraform

export AWS_PROFILE=terraform
```

#### Configure kubectl to allow us to connect to EKS cluster

In terraform configuration we output configuration file for kubectl

```sh
terraform output kubeconfig
```

#### Add output of "terraform output kubeconfig" to ~/.kube/config-devel

```sh
terraform output kubeconfig > ~/.kube/config-devel

export KUBECONFIG=$KUBECONFIG:~/.kube/config-devel
```

#### Verify kubectl connectivity

```sh
kubectl get namespaces

kubectl get services
```

#### Second part we need to allow EKS to add nodes by running configmap

```sh
terraform output config_map_aws_auth > yaml/config_map_aws_auth.yaml

kubectl apply -f yaml/config_map_aws_auth.yaml
```

#### Now you should be able to see nodes

```sh
kubectl get nodes
```

[![asciicast](https://asciinema.org/a/195818.png)](https://asciinema.org/a/195818)

## Working with terraform on EKS

### Deploy the [Kubernetes Dashboard](https://github.com/kubernetes/dashboard)

#### Deploy the Kubernetes dashboard

```sh
kubectl apply -f \
https://raw.githubusercontent.com/kubernetes/dashboard/master/src/deploy/recommended/kubernetes-dashboard.yaml
```

### Create an eks-admin Service Account and Cluster Role Binding

#### Apply the service account to your cluster

```sh
kubectl apply -f yaml/eks-admin-service-account.yaml
```

#### Apply the cluster role binding to your cluster

```sh
kubectl apply -f yaml/eks-admin-cluster-role-binding.yaml
```

### Connect to the Dashboard

```sh
kubectl -n kube-system describe secret $(kubectl -n kube-system get secret | grep eks-admin | awk '{print $1}')

kubectl proxy
```

> NOTE: Open the link with a web browser to access the dashboard endpoint: <http://localhost:8001/api/v1/namespaces/kube-system/services/https:kubernetes-dashboard:/proxy/>

> NOTE: Choose Token and paste output from the previous command into the Token field

[![asciicast](https://asciinema.org/a/195823.png)](https://asciinema.org/a/195823)

## Rolling back all changes

### Destroy all terraform created infrastructure

```sh
terraform destroy -auto-approve
```

[![asciicast](https://asciinema.org/a/195827.png)](https://asciinema.org/a/195827)

### Removing S3 bucket, IAM roles and terraform account

```sh
export AWS_PROFILE=default

aws s3 rm s3://catalyst-dev-tfstate --recursive

aws s3api put-bucket-versioning --bucket catalyst-dev-tfstate --versioning-configuration Status=Suspended

aws s3api delete-objects --bucket catalyst-dev-tfstate --delete \
"$(aws s3api list-object-versions --bucket catalyst-dev-tfstate | \
jq '{Objects: [.Versions[] | {Key:.Key, VersionId : .VersionId}], Quiet: false}')"

aws s3 rb s3://catalyst-dev-tfstate --force

aws iam detach-user-policy --user-name terraform --policy-arn arn:aws:iam::aws:policy/AdministratorAccess

aws iam list-access-keys --user-name terraform  --query 'AccessKeyMetadata[*].{ID:AccessKeyId}' --output text

aws iam delete-access-key --user-name terraform --access-key-id OUT_KEY

aws iam delete-user --user-name terraform
```
