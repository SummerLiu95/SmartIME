"use client"

import { motion, HTMLMotionProps } from "framer-motion"
import { cn } from "@/lib/utils"

interface ScaleInProps extends HTMLMotionProps<"div"> {
  delay?: number
  duration?: number
}

export function ScaleIn({ 
  children, 
  className, 
  delay = 0, 
  duration = 0.4,
  ...props 
}: ScaleInProps) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{ 
        duration, 
        delay, 
        ease: [0.21, 0.47, 0.32, 0.98] 
      }}
      className={cn(className)}
      {...props}
    >
      {children}
    </motion.div>
  )
}
