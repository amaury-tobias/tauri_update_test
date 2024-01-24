<script setup lang="ts">
import { ref } from "vue"
import { getVersion } from "@tauri-apps/api/app"
import { invoke } from "@tauri-apps/api/tauri"

const appVersion = ref('')
getVersion()
  .then((version) => {
    appVersion.value = version
  })

const pdfData = ref()
invoke('plugin:drive|get_file_by_id', {
  fileId: '10og74Ba3-9WVYp-apnq4U5u_vbebCxwC',
})
  .then(result => {
    pdfData.value = result
  })
  .catch(err => console.error(err))
</script>

<template>
  <div>
    <p>{{ appVersion }}</p>
    <pre>{{ pdfData }}</pre>
  </div>
</template>
