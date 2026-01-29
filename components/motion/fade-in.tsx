"use client"

import { motion, HTMLMotionProps } from "framer-motion"
import { cn } from "@/lib/utils"

interface FadeInProps extends HTMLMotionProps<"div"> {
  delay?: number
  duration?: number
  blur?: boolean
}

export function FadeIn({ 
  children, 
  className, 
  delay = 0, 
  duration = 0.5, 
  blur = false,
  ...props 
}: FadeInProps) {
  return (
    <motion.div
      initial={{ opacity: 0, filter: blur ? "blur(10px)" : "none" }}
      animate={{ opacity: 1, filter: "blur(0px)" }}
      exit={{ opacity: 0, filter: blur ? "blur(10px)" : "none" }}
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
