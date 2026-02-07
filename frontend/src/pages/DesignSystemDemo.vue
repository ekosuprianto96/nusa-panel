<script setup lang="ts">
/**
 * Design System Demo Page
 * 
 * Halaman contoh penggunaan Design System
 * Demonstrasi semua components dan theme switching
 */

import { ref } from 'vue'
import { 
  Button, 
  Input, 
  Card, 
  CardHeader, 
  CardTitle, 
  CardDescription, 
  CardContent, 
  CardFooter,
  Badge,
  Textarea
} from '@/components/ui'
import { useTheme } from '@/design-system'
import { Sun, Moon, Palette, Check, X, AlertCircle, Info } from 'lucide-vue-next'

// Theme composable
const { isDark, toggleColorMode, setTheme, availableThemes } = useTheme()


// Form state
const email = ref('')
const password = ref('')
const message = ref('')
const isLoading = ref(false)

// Simulate loading
const handleSubmit = () => {
  isLoading.value = true
  setTimeout(() => {
    isLoading.value = false
  }, 2000)
}
</script>

<template>
  <div class="min-h-screen bg-background text-foreground p-8">
    <div class="max-w-6xl mx-auto space-y-12">
      
      <!-- Header -->
      <header class="text-center space-y-4">
        <h1 class="text-4xl font-bold text-gradient">
          Design System Demo
        </h1>
        <p class="text-muted-foreground text-lg">
          Modular & Dynamic Design System for Vue.js
        </p>
        
        <!-- Theme Switcher -->
        <div class="flex items-center justify-center gap-4">
          <Button 
            :variant="isDark ? 'secondary' : 'default'" 
            @click="toggleColorMode"
          >
            <Sun v-if="isDark" class="w-4 h-4" />
            <Moon v-else class="w-4 h-4" />
            {{ isDark ? 'Light Mode' : 'Dark Mode' }}
          </Button>
          
          <div class="flex gap-2">
            <Button
              v-for="theme in availableThemes"
              :key="theme"
              variant="outline"
              size="sm"
              @click="setTheme(theme)"
            >
              <Palette class="w-3 h-3" />
              {{ theme }}
            </Button>
          </div>
        </div>
      </header>

      <!-- Button Variants -->
      <section>
        <Card>
          <CardHeader>
            <CardTitle>Button Component</CardTitle>
            <CardDescription>
              Berbagai variant dan size untuk button
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-6">
            <!-- Variants -->
            <div>
              <h4 class="text-sm font-medium mb-3">Variants</h4>
              <div class="flex flex-wrap gap-3">
                <Button variant="default">Default</Button>
                <Button variant="secondary">Secondary</Button>
                <Button variant="outline">Outline</Button>
                <Button variant="ghost">Ghost</Button>
                <Button variant="link">Link</Button>
                <Button variant="destructive">Destructive</Button>
                <Button variant="success">Success</Button>
                <Button variant="warning">Warning</Button>
              </div>
            </div>
            
            <!-- Sizes -->
            <div>
              <h4 class="text-sm font-medium mb-3">Sizes</h4>
              <div class="flex items-center gap-3">
                <Button size="sm">Small</Button>
                <Button size="default">Default</Button>
                <Button size="lg">Large</Button>
                <Button size="xl">Extra Large</Button>
                <Button size="icon"><Check class="w-4 h-4" /></Button>
              </div>
            </div>
            
            <!-- States -->
            <div>
              <h4 class="text-sm font-medium mb-3">States</h4>
              <div class="flex items-center gap-3">
                <Button :loading="isLoading" @click="handleSubmit">
                  {{ isLoading ? 'Loading...' : 'Click to Load' }}
                </Button>
                <Button disabled>Disabled</Button>
              </div>
            </div>
          </CardContent>
        </Card>
      </section>

      <!-- Input & Form -->
      <section>
        <Card>
          <CardHeader>
            <CardTitle>Form Components</CardTitle>
            <CardDescription>
              Input, Textarea dengan variant system
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form class="space-y-4 max-w-md" @submit.prevent>
              <!-- Default Input -->
              <div class="space-y-2">
                <label class="text-sm font-medium">Email</label>
                <Input 
                  v-model="email" 
                  type="email" 
                  placeholder="Enter your email"
                />
              </div>
              
              <!-- Error Input -->
              <div class="space-y-2">
                <label class="text-sm font-medium">Password (Error State)</label>
                <Input 
                  v-model="password" 
                  type="password" 
                  variant="error"
                  placeholder="Enter password"
                />
                <p class="text-xs text-destructive">Password is required</p>
              </div>
              
              <!-- Success Input -->
              <div class="space-y-2">
                <label class="text-sm font-medium">Username (Success State)</label>
                <Input 
                  variant="success"
                  placeholder="Username available"
                  default-value="johndoe"
                />
              </div>
              
              <!-- Textarea -->
              <div class="space-y-2">
                <label class="text-sm font-medium">Message</label>
                <Textarea 
                  v-model="message" 
                  placeholder="Type your message..."
                  :rows="4"
                />
              </div>
              
              <!-- Input Sizes -->
              <div class="space-y-2">
                <label class="text-sm font-medium">Input Sizes</label>
                <div class="space-y-2">
                  <Input size="sm" placeholder="Small input" />
                  <Input size="default" placeholder="Default input" />
                  <Input size="lg" placeholder="Large input" />
                </div>
              </div>
            </form>
          </CardContent>
          <CardFooter class="gap-3">
            <Button type="submit" :loading="isLoading" @click="handleSubmit">
              Submit Form
            </Button>
            <Button variant="outline">Cancel</Button>
          </CardFooter>
        </Card>
      </section>

      <!-- Badge Variants -->
      <section>
        <Card>
          <CardHeader>
            <CardTitle>Badge Component</CardTitle>
            <CardDescription>
              Status indicators dengan berbagai variant
            </CardDescription>
          </CardHeader>
          <CardContent class="space-y-4">
            <!-- Variants -->
            <div>
              <h4 class="text-sm font-medium mb-3">Variants</h4>
              <div class="flex flex-wrap gap-2">
                <Badge variant="default">Default</Badge>
                <Badge variant="secondary">Secondary</Badge>
                <Badge variant="outline">Outline</Badge>
                <Badge variant="destructive">
                  <X class="w-3 h-3 mr-1" />
                  Error
                </Badge>
                <Badge variant="success">
                  <Check class="w-3 h-3 mr-1" />
                  Success
                </Badge>
                <Badge variant="warning">
                  <AlertCircle class="w-3 h-3 mr-1" />
                  Warning
                </Badge>
                <Badge variant="info">
                  <Info class="w-3 h-3 mr-1" />
                  Info
                </Badge>
              </div>
            </div>
            
            <!-- Sizes -->
            <div>
              <h4 class="text-sm font-medium mb-3">Sizes</h4>
              <div class="flex items-center gap-2">
                <Badge size="sm">Small</Badge>
                <Badge size="default">Default</Badge>
                <Badge size="lg">Large</Badge>
              </div>
            </div>
          </CardContent>
        </Card>
      </section>

      <!-- Card Variations -->
      <section>
        <h2 class="text-2xl font-semibold mb-4">Card Variations</h2>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          <!-- Standard Card -->
          <Card>
            <CardHeader>
              <CardTitle>Standard Card</CardTitle>
              <CardDescription>Basic card layout</CardDescription>
            </CardHeader>
            <CardContent>
              <p class="text-sm text-muted-foreground">
                This is a standard card with header, content, and footer sections.
              </p>
            </CardContent>
            <CardFooter>
              <Button size="sm">Action</Button>
            </CardFooter>
          </Card>
          
          <!-- Hoverable Card -->
          <Card hoverable>
            <CardHeader>
              <CardTitle>Hoverable Card</CardTitle>
              <CardDescription>With hover effect</CardDescription>
            </CardHeader>
            <CardContent>
              <p class="text-sm text-muted-foreground">
                Hover over this card to see the shadow effect.
              </p>
            </CardContent>
          </Card>
          
          <!-- Clickable Card -->
          <Card clickable>
            <CardHeader>
              <CardTitle>Clickable Card</CardTitle>
              <CardDescription>Interactive card</CardDescription>
            </CardHeader>
            <CardContent>
              <p class="text-sm text-muted-foreground">
                This card has a clickable appearance with cursor pointer.
              </p>
            </CardContent>
          </Card>
        </div>
      </section>

      <!-- Token Examples -->
      <section>
        <Card>
          <CardHeader>
            <CardTitle>Design Tokens</CardTitle>
            <CardDescription>
              CSS Variables yang dihasilkan dari theme.config.ts
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div class="space-y-2">
                <div 
                  class="w-full h-16 rounded-lg border"
                  :style="{ backgroundColor: `hsl(var(--primary))` }"
                />
                <p class="text-xs font-mono">--primary</p>
              </div>
              <div class="space-y-2">
                <div 
                  class="w-full h-16 rounded-lg border"
                  :style="{ backgroundColor: `hsl(var(--secondary))` }"
                />
                <p class="text-xs font-mono">--secondary</p>
              </div>
              <div class="space-y-2">
                <div 
                  class="w-full h-16 rounded-lg border"
                  :style="{ backgroundColor: `hsl(var(--destructive))` }"
                />
                <p class="text-xs font-mono">--destructive</p>
              </div>
              <div class="space-y-2">
                <div 
                  class="w-full h-16 rounded-lg border"
                  :style="{ backgroundColor: `hsl(var(--success))` }"
                />
                <p class="text-xs font-mono">--success</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </section>

      <!-- Code Example -->
      <section>
        <Card>
          <CardHeader>
            <CardTitle>Usage Example</CardTitle>
            <CardDescription>
              Contoh penggunaan dalam kode
            </CardDescription>
          </CardHeader>
          <CardContent>
            <pre class="bg-muted p-4 rounded-lg overflow-x-auto text-sm"><code>// Import components
import { Button, Input, Card } from '@/components/ui'
import { useTheme } from '@/design-system'

// Use theme composable
const { isDark, toggleColorMode } = useTheme()

// In template
&lt;Button variant="primary" size="lg"&gt;
  Click Me
&lt;/Button&gt;

&lt;Input v-model="email" variant="error" /&gt;

&lt;Card hoverable&gt;
  &lt;CardHeader&gt;...&lt;/CardHeader&gt;
  &lt;CardContent&gt;...&lt;/CardContent&gt;
&lt;/Card&gt;</code></pre>
          </CardContent>
        </Card>
      </section>

    </div>
  </div>
</template>

<style>
.text-gradient {
  @apply bg-clip-text text-transparent bg-gradient-to-r from-blue-600 to-indigo-600;
}

.dark .text-gradient {
  @apply from-blue-400 to-indigo-400;
}
</style>
