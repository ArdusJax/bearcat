import com.concur.*;

testHelper      = new com.concur.test.TestHelper()
concurPipeline  = new ConcurCommands()
concurUtil      = new Util()

workflowDoc = '''
title: Cargo
overview: Execute any Cargo task.
additional_resources:
  - name: Cargo Official
    url: https://doc.rust-lang.org/cargo/index.html
tools:
  - type: String
    name: buildImage
    required: true
    section: cargo
    description: Docker image containing Cargo and any other necessary tools for the project to build
  - type: String
    name: dockerOpts
    section: cargo
    required: false
    description: Options to pass to Docker container run command
  - type: String
    name: task
    section: cargo
    description: The name of the task to execute, multiple tasks can be separated by a space
    default: build
  - type: List
    name: extraArgs
    section: cargo
    description: Any additional arguments to apply to the Cargo task
full_example: |
  pipelines:
    tools:
      github:
        patterns:
          feature: .+
    tools:
      cargo:
        buildImage: cargo:1.0.0
    branches:
      feature:
        steps:
          - cargo:
            # Simple
            - task: compile
            # Advanced
            - task:
                # Single task
                name: build 
                # Multiple tasks
                name: "test build publish"
                credentialList:
                  - name: MYUSERPASS # Create MYUSERPASS_USERNAME & MYUSERPASS_PASSWORD as environment vars for Cargo
                    criteria:
                      username: myuser # Credential username in Buildhub
                  - name: MYSECRET # Create MYSECRET as environment var for Cargo
                    criteria:
                      description: my_secret # Credential description in Buildhub
'''

/*
description: Execute Cargo tasks
parameters:
  - type: String
    name: buildImage
    required: true
    description: Docker image containing Cargo and any other necessary tools for the project to build
  - type: String
    name: dockerOpts
    required: false
    description: Options to pass to Docker container run command
  - type: String
    name: name
    description: The name of the task to execute, multiple tasks can be separated by a space
    default: build
  - type: List
    name: extraArgs
    description: Any additional arguments to apply to the Cargo task
  - type: List
    name: credentialList
    description: Credentials to inject during task execution
example: |
  branches:
    feature:
      steps:
        - cargo:
            # Simple
            - task: build
            # Advanced
            - task:
                # Single task
                name: build
                # Multiple tasks
                name: "test build publish"
                credentialList:
                  - name: MYUSERPASS # Create MYUSERPASS_USERNAME & MYUSERPASS_PASSWORD as environment vars for Cargo
                    criteria:
                      username: myuser # Credential username in Buildhub
                  - name: MYSECRET # Create MYSECRET as environment var for Cargo
                    criteria:
                      description: my_secret # Credential description in Buildhub
 */
public task(Map yml, Object args) {
  String dockerImage   = yml.tools?.cargo?.buildImage
  String cargoBinary = yml.tools?.cargo?.binary ?: "cargo"
  def extraArgs        = yml.tools?.cargo?.extraArgs
  String dockerOpts    = yml.tools?.cargo?.dockerOpts ?: ""
  List credentialList   = yml.tools?.cargo?.credentialList ?: []

  assert args  : "Workflows :: cargo :: task :: No arguments provided to this step; step does not have a default task so you must provide one. ${Constants.Strings.WORKFLOWS_REFER_TO_DOCUMENTATION}"

  def cargoTask
  if (args instanceof String) {
    cargoTask = "${cargoBinary} ${args}"
  } else if (args instanceof List) {
    cargoTask = "${cargoBinary} ${args.join(' ')}"
  } else if (args instanceof Map) {
    dockerImage       = args?.buildImage     ?: dockerImage
    dockerOpts        = args?.dockerOpts     ?: dockerOpts
    extraArgs         = args?.extraArgs      ?: extraArgs
    String taskName   = args?.name           ?: "build"
    credentialList    = args?.credentialList ?: credentialList

    cargoTask = "${binary} ${taskName}"
  }

  if (extraArgs) {
    if (extraArgs instanceof List) {
      extraArgs = extraArgs.join(" ")
    } else if (extraArgs instanceof Map) {
      extraArgs = extraArgs.collect { "${it.key}=${it.value}" }.join(' ')
    }
    cargoTask = "${cargoTask} ${extraArgs}"
  }

  assert dockerImage  : "Workflows :: cargo :: task :: No [buildImage] provided in [tools.cargo] or as a parameter to this step."

  concurPipeline.debugPrint('Workflows :: cargo :: task', [
    'args'           : args,
    'extraArgs'      : extraArgs,
    'buildImage'    : dockerImage,
    'dockerOpts'     : dockerOpts,
    'binary'   : cargoBinary,
    'credentialList' : credentialList
  ])

  if(cargoTask) {
    def creds = []
    if (credentialList) {
      creds = concurPipeline.getWithCredentialList(credentialList)
    }

    docker.image(dockerImage).inside("--entrypoint='' ${dockerOpts}") {
      withCredentials(creds) {
        sh cargoTask
      }
    }
  }
}

/*
  ******************************* REQUIRED FUNCTIONS *************************************
  This area is for functions that are required to be included in a workflow.
  Currently this includes `def tests(Map yml)` and `public getStageName(Map yml, Map args, String stepName)`
 */

public getStageName(Map yml, Object args, String stepName) {
  String baseName = "cargo: $stepName"
  String cargoTask
  if (args instanceof String) {
    cargoTask = args
  } else if (args instanceof List) {
    cargoTask = args.join(' ')
  } else if (args instanceof Map) {
    cargoTask  = args?.name ?: "build"
  }

  switch (stepName) {
    case 'task':
      return "$baseName: $cargoTask"
    default: return baseName
  }
}

def tests(yml) {
  String workflowName = 'cargo'
  println testHelper.header("Testing ${workflowName}.groovy...")

  // Mock for the pipelines.yml used for testing
  def cargoYml = concurUtil.mustacheReplaceAll(concurUtil.parseYAML(readFile(yml.testing.cargoTest.cargoYaml))).pipelines

  def testCargoBuild       = readFile yml.testing.cargoTest.build
  def testCargoProperties  = readFile yml.testing.cargoTest.props

  // Method test
  boolean passed = true
  try {
    println testHelper.debug("Running Cargo init...")
    task(cargoYml, 'init')
    println testHelper.debug("Removing initial build.cargo...")
    sh 'rm build.cargo'
    println testHelper.debug("Creating [build.cargo]...")
    writeFile encoding: 'utf-8', file: 'build.cargo', text: testCargoBuild
    println testHelper.debug("Creating [cargo.properties]...")
    writeFile encoding: 'utf-8', file: 'cargo.properties', text: testCargoProperties
    println testHelper.debug("Calling [cargo] function...")
    task(cargoYml, ['clean', 'build'])
    println testHelper.debug("Calling [cargo] function with args...")
    task(cargoYml, ['name':'workflowsHello'])

    println testHelper.debug("Testing $workflowName [getStageName]...")
    List testCases = [
      [
        'args'    : [:],
        'yml'     : cargoYml,
        'expected': 'cargo: task: build',
        'stepName': 'task'
      ],
      [
        'args'    : [name:'test'],
        'yml'     : cargoYml,
        'expected': 'cargo: task: test',
        'stepName': 'task'
      ],
      [
        'args'    : 'test',
        'yml'     : cargoYml,
        'expected': 'cargo: task: test',
        'stepName': 'task'
      ],
      [
        'args'    : ['test', 'trap'],
        'yml'     : cargoYml,
        'expected': 'cargo: task: test trap',
        'stepName': 'task'
      ],
    ]
    testCases.each { tc ->
      String stageName = getStageName(tc.yml, tc.args, tc.stepName)
      if (stageName != tc.expected) {
        error("Workflows :: $workflowName :: tests :: Failed to match expected result, got [$stageName] expected [${tc.expected}]")
      }
    }
  } catch (e) {
    passed = false
    testHelper.fail("""|Errors with ${workflowName}.groovy
                       |----------------------------
                       |$e""".stripMargin())
  } finally {
    if (passed) {
      println testHelper.success("Testing for ${workflowName}.groovy passed")
      passedTests.add(workflowName)
    } else {
      println testHelper.fail("${workflowName}.groovy Testing failed")
      failedTests.add(workflowName)
    }
  }
}
return this;
